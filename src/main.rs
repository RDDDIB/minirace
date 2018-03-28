extern crate mechanics;

use mechanics::*;
use std::{io, thread, time};
use std::cmp::min;

/// Return a coloured symbol.
///
/// Colour a symbol based on the given turn ranking.
///
/// # Arguments
///
/// * `typ` - turn ranking
/// * `sym` - the symbol to use
fn sym(typ: &isize, sym: char, bright: bool) -> String {
    let bright = match bright {
        true => ";1",
        false => ""
    };
    format!("{}{}\u{1b}[0m", match *typ {
        1 => format!("\u{1b}[37{}m", bright),
        2 => format!("\u{1b}[34{}m", bright),
        3 => format!("\u{1b}[36{}m", bright),
        4 => format!("\u{1b}[35{}m", bright),
        5 => format!("\u{1b}[33{}m", bright),
        6 => format!("\u{1b}[31{}m", bright),
        _ => "".to_string()
    }, sym)
}

/// Place an icon with a speed arrow.
///
/// Put the first letter of the racer's name at the racer's current 
/// position along the lap along with an arrow.
/// The arrow indicates the racer's range and the next turn through its
/// length and colour, respectively.
///
/// # Arguments
///
/// * `path` - the lap
/// * `racers` - a list of racers
///
/// # Example
///
///     A-->
///      B>
///   C->
fn show_pos(path: &Vec<isize>, racers: &[Racer]) -> String {
    let mut possbar = Vec::new();
    for player in racers {
        let mut poss = Vec::new();
        let bg = |x| sym(&path[x], '#', false);
        for i in 0..min(path.len(), player.position.turn) {
            poss.push(bg(i))
        }
        poss.push(format!("{}", player.shortname()));
        let arm = |x| sym(&player.next_turn(path), x, true);
        for _ in 0..player.move_range() - 1 {
            poss.push(arm('-'));
        }
        poss.push(arm('>'));
        let start = player.position.turn + player.move_range() as usize;
        for i in start + 1..path.len() {
            poss.push(bg(i));
        }
        possbar.push(poss.join(""));
    }
    possbar.join("\n")
}

/// Calculate total turns completed.
///
/// laps * size(lap) + turn
///
/// # Arguments
///
/// * `pos` - current position
/// * `size` - lap size
fn sum_pos(pos: &Position, size: usize) -> usize {
    pos.lap * size + pos.turn
}

/// Progress bar
///
/// # Arguments
///
/// * `pos` - current position
/// * `laps` - total laps
/// * `size` - lap size
///
/// # Example
///
/// lap 2 turn 1 gives 
/// |#########|#--------|
fn progress(player: &Racer, laps: usize, size: usize) -> String {
    let mut msg = String::new();
    let thresh = sum_pos(&player.position, size);
    let col = match player.alive {true => "34;1", false => "31"};

    for i in 0..laps {
        msg.push('|');
        for j in 0..size {
            if i * size + j <= thresh {
                msg += &*format!("\u{1b}[{}m#\u{1b}[0m", col);
                //println!("{} - {}", thresh, i * size + j);
            } else {
                msg.push('-');
            }
        }
    }
    msg.push('|');
    msg
}

fn main() {
    // The circuit (turn rankings)
    let path = vec![1, 2, 1, 1, 1, 2, 1, 1, 3, 1, 1, 3, 1, 1, 5, 1, 1, 4, 1, 3, 1, 1, 3, 1, 1, 1, 3, 1];
    // The circuit (colourized)
    let syms: Vec<String> = path.iter().map(|x| format!("{}", sym(x, '#', true))).collect();

    // Clear screen
    println!("\u{1b}[2J");
    // Ask for name
    println!("Pick a name!");
    let mut main_name = String::new();
    io::stdin().read_line(&mut main_name).expect("Failed to read line");
    main_name.pop();
    // Clear screen
    println!("\u{1b}[2J");

    // Gen racers
    let mut racers = [Racer::new(main_name.clone(), AI::Human),
                      Racer::new(String::from("Roger"), AI::NPC(Brain::Slug)),
                      Racer::new(String::from("Brian"), AI::NPC(Brain::Nocombat)),
                      Racer::new(String::from("Nick"), AI::NPC(Brain::Nocombat)),
                      Racer::new(String::from("Lionel"), AI::NPC(Brain::Nocombat)),
                      Racer::new(String::from("Max"), AI::NPC(Brain::Aggressive))];

    // Uncomment to let the NPCs run on their own
    // racers[0].alive = false;

    // Game
    // Initialize
    // Total number of laps
    let num_laps = 3;

    // Racer info line
    let info = |player: &Racer| format!("{}\tHP \u{1b}[31;1m{}\u{1b}[0m\tSpeed \u{1b}[32;1m{}\u{1b}[0m\tLap \u{1b}[33m{}\u{1b}[0m\tSteps Left \u{1b}[33m{}\u{1b}[0m\n{}\n",
                                player.name,
                                player.hp,
                                speed_adj(&player.speed),
                                player.position.lap + 1,
                                path.len() - player.position.turn,
                                progress(&player, num_laps, path.len()));

    // Calculate HP for each racer
    for player in &mut racers {
        player.reset_hp(&path);
    }

    // While at least one racer is still in and no-one has crossed the finish,
    while racers.iter().any(|x| x.alive) && racers.iter().all(|x| x.position.lap < num_laps) {
        // Print lap
        // Print racer positions/ranges
        let mut disr: Vec<String> = Vec::new();
        for player in &mut racers {
            disr.push(player.get_log().join("\n"));
        }
        disr.push(syms.join(""));
        disr.push(show_pos(&path, &racers));
        let disr = disr.join("\n");
        println!("\u{1b}[2J{}\n{}\n", disr, syms.join(""));

        // Racer info line
        for player in &mut racers {
            println!("{}", info(player));
        }
        println!("\n");

        for player in &mut racers {
            if player.alive {
                match player.ai {
                    AI::Human => {
                        let can_heal = player.move_range() as usize + player.position.turn >= path.len();
                        // Give console player a choice to change speed
                        println!("{}\n1 - Decrease Speed\n2 - Increase Speed\n3 - Keep Your Speed{}",
                                 player.name,
                                 if can_heal {"\nH - Pitstop (resets your health and speed)"} else {" "});
                        let mut decision = String::new();
                        println!();
                        io::stdin().read_line(&mut decision).expect("Failed to read line");
                        decision.pop();
                        match &*decision {
                            "1" => player.slow_down(),
                            "2" => player.speed_up(),
                            "H" => if can_heal {
                                player.reset_hp(&path);
                                player.speed = -2},
                            _   => ()
                        };
                    },
                    AI::NPC(_) => () // player.ai_move(&path)
                };

                // Attempt turn
                player.make_turn(&path);
            }

            // Check for win
            if player.position.lap >= num_laps {
                println!("\u{1b}[2J{} wins!", player.name);
                break;
            }
        }

        if racers.to_vec().iter()
            .filter(|x| match x.ai {AI::Human => true, _ => false})
                .all(|y| !y.alive) {
                    thread::sleep(time::Duration::from_millis(500));
                }
    }

    println!("Leaderboard:");

    let mut wins = racers.to_vec();
    wins.sort_by(|a, b| sum_pos(&a.position, path.len()).cmp(&sum_pos(&b.position, path.len())));
    wins.reverse();
    for (i, player) in wins.iter().enumerate() {
        println!("{} - {}", i + 1, player.name);
    }
    thread::sleep(time::Duration::from_millis(2000));
}
