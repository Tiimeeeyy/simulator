use std::path::PathBuf;
use std::sync::Arc;

use ygopro_bot::{GenericAgent, TearlamentsPolicy};
use ygopro_core::{Duel, DuelConfig, ScriptLoader};
use ygopro_data::CardDatabase;
use ygopro_host::{DuelSession, PlayerController, StepOutcome};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_bot_self_play_duel() {
    let root = repo_root();
    let cdb_path = root.join("data").join("cards.cdb");
    let scripts_dir = root.join("data").join("scripts");

    assert!(cdb_path.exists());
    assert!(scripts_dir.exists());

    let db = Arc::new(CardDatabase::open(&cdb_path).expect("failed to open cards.cdb"));
    let script_loader = Arc::new(ScriptLoader::new().with_dir(&scripts_dir));

    let config = DuelConfig::default();
    let mut duel = Duel::create(config, Arc::clone(&db), Arc::clone(&script_loader))
        .expect("failed to create duel");

    let deck = ygopro_data::builtin::tearlaments_ishizu_deck();
    duel.load_deck(0, &deck);
    duel.load_deck(1, &deck);

    let p0 = PlayerController::Bot(Box::new(TearlamentsPolicy::new(0)));
    let p1 = PlayerController::Bot(Box::new(GenericAgent::new(1)));

    let mut session = DuelSession::new(duel, Arc::clone(&db), p0, p1);

    let mut bot_actions = 0;
    let max_steps = 100;

    for step_i in 0..max_steps {
        let outcome = session.step().expect("step failed");
        match outcome {
            StepOutcome::Continue(msgs) => {
                println!("Step {step_i}: Continue with {} msgs: {:?}", msgs.len(), msgs.first());
            }
            StepOutcome::BotActed { player, response, messages } => {
                bot_actions += 1;
                println!("Step {step_i}: Bot {player} responded: {response:?} (msgs: {})", messages.len());
            }
            StepOutcome::AwaitingHuman { .. } => {
                panic!("unexpected human prompt in all-bot match");
            }
            StepOutcome::Ended { winner, .. } => {
                println!("Step {step_i}: Duel ended! Winner: {winner:?}");
                break;
            }
        }
    }

    assert!(bot_actions > 0, "expected bots to perform actions, got {bot_actions}");
    println!("Total bot actions executed: {bot_actions}");
}
