//! Duel coordination session managing players, bots, and engine stepping.

use std::sync::Arc;
use ygopro_bot::DuelAgent;
use ygopro_core::{CoreMessage, Duel, PlayerResponse, ProcessResult};
use ygopro_data::CardDatabase;

use crate::error::{HostError, Result};

/// Controller driving a player slot.
pub enum PlayerController {
    /// In-process native Rust bot (e.g. `TearlamentsPolicy` or `GenericAgent`).
    Bot(Box<dyn DuelAgent>),
    /// Interactive human player controlled via UI / input.
    Human { name: String },
}

/// The outcome of stepping the session.
#[derive(Debug, Clone)]
pub enum StepOutcome {
    /// Engine processed internal events and can continue.
    Continue(Vec<CoreMessage>),
    /// A bot automatically responded to a prompt.
    BotActed {
        player: u8,
        response: PlayerResponse,
        messages: Vec<CoreMessage>,
    },
    /// Duel is paused waiting for human player input.
    AwaitingHuman {
        player: u8,
        prompt: Box<CoreMessage>,
        messages: Vec<CoreMessage>,
    },
    /// Duel has ended.
    Ended {
        winner: Option<u8>,
        messages: Vec<CoreMessage>,
    },
}

/// A coordinated duel session.
pub struct DuelSession {
    duel: Duel,
    db: Arc<CardDatabase>,
    p0: PlayerController,
    p1: PlayerController,
    winner: Option<u8>,
}

impl DuelSession {
    pub fn new(
        mut duel: Duel,
        db: Arc<CardDatabase>,
        p0: PlayerController,
        p1: PlayerController,
    ) -> Self {
        duel.start();
        Self {
            duel,
            db,
            p0,
            p1,
            winner: None,
        }
    }

    pub fn winner(&self) -> Option<u8> {
        self.winner
    }

    pub fn duel_mut(&mut self) -> &mut Duel {
        &mut self.duel
    }

    /// Single step of the duel loop.
    pub fn step(&mut self) -> Result<StepOutcome> {
        if self.duel.is_ended() {
            return Ok(StepOutcome::Ended {
                winner: self.winner,
                messages: Vec::new(),
            });
        }

        let res = self.duel.process()?;
        let msgs = self.duel.get_messages()?;

        // Track winner if win message present
        for m in &msgs {
            if let CoreMessage::Win { player, .. } = m {
                self.winner = Some(*player);
            }
        }

        // Broadcast messages to bot agents
        for m in &msgs {
            if let PlayerController::Bot(ref mut bot) = self.p0 {
                bot.on_message(m);
            }
            if let PlayerController::Bot(ref mut bot) = self.p1 {
                bot.on_message(m);
            }
        }

        match res {
            ProcessResult::End => {
                Ok(StepOutcome::Ended {
                    winner: self.winner,
                    messages: msgs,
                })
            }
            ProcessResult::Continue => {
                Ok(StepOutcome::Continue(msgs))
            }
            ProcessResult::Awaiting => {
                // Find the active prompt message
                let prompt = msgs.iter().rev().find(|m| m.is_prompt()).cloned();
                if let Some(prompt_msg) = prompt {
                    let player = prompt_msg.prompt_player().unwrap_or(0);
                    let controller = if player == 0 { &mut self.p0 } else { &mut self.p1 };

                    match controller {
                        PlayerController::Bot(bot) => {
                            let resp = bot.respond(&prompt_msg, &self.db);
                            self.duel.set_response(&resp);
                            Ok(StepOutcome::BotActed {
                                player,
                                response: resp,
                                messages: msgs,
                            })
                        }
                        PlayerController::Human { .. } => {
                            Ok(StepOutcome::AwaitingHuman {
                                player,
                                prompt: Box::new(prompt_msg),
                                messages: msgs,
                            })
                        }
                    }
                } else {
                    Ok(StepOutcome::Continue(msgs))
                }
            }
        }
    }

    /// Advance the duel until a human action is required, or until the duel ends.
    pub fn run_until_human_prompt_or_end(&mut self) -> Result<StepOutcome> {
        let mut accumulated_msgs = Vec::new();

        loop {
            let outcome = self.step()?;
            match outcome {
                StepOutcome::Continue(msgs) => {
                    accumulated_msgs.extend(msgs);
                }
                StepOutcome::BotActed { messages, .. } => {
                    accumulated_msgs.extend(messages);
                }
                StepOutcome::AwaitingHuman {
                    player,
                    prompt,
                    messages,
                } => {
                    accumulated_msgs.extend(messages);
                    return Ok(StepOutcome::AwaitingHuman {
                        player,
                        prompt,
                        messages: accumulated_msgs,
                    });
                }
                StepOutcome::Ended { winner, messages } => {
                    accumulated_msgs.extend(messages);
                    return Ok(StepOutcome::Ended {
                        winner,
                        messages: accumulated_msgs,
                    });
                }
            }
        }
    }

    /// Submit a response on behalf of a human player.
    pub fn submit_human_response(&mut self, _player: u8, response: &PlayerResponse) -> Result<()> {
        if self.duel.is_ended() {
            return Err(HostError::AlreadyEnded);
        }
        self.duel.set_response(response);
        Ok(())
    }
}
