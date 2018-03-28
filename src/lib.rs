#![allow(dead_code)]
extern crate rand;

use std::fmt;
use std::fmt::Display;
use rand::{thread_rng, Rng};

/// Roll a d6
pub fn roll() -> usize {
    let mut rng = thread_rng();
    rng.gen_range(0, 7)
}

#[derive(Debug, Clone)]
pub enum AI {
    Human,
    NPC(Brain)
}

#[derive(Debug, Clone)]
pub enum Brain {
    Nocombat,
    Aggressive,
    Beast,
    Deathwish,
    Lurker,
    Slug
}

/// A position in laps and turns.
#[derive(Debug, Clone)]
pub struct Position {
    /// The lap
    pub lap: usize,
    /// The index of the current turn in the lap
    pub turn: usize
}

/// Vroom!
#[derive(Debug, Clone)]
pub struct Racer {
    /// The name of the valiant driver
    pub name: String,
    /// The health of the car
    pub hp: isize,
    /// The current position
    pub position: Position,
    /// The speed ranking
    pub speed: isize,
    /// Don't Crash!
    pub alive: bool,
    /// Controller
    pub ai: AI,
    /// Turn Index
    pub turn: usize,
    /// Log
    pub log: Vec<Vec<String>>
}

impl Display for Racer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} is {} through lap {}!",
               self.name,
               speed_adj(&self.speed),
               self.position.lap + 1)
    }
}

impl Racer {
    /// Create a new Racer with a given name.
    ///
    /// Place a new racer at the start of the circuit at the slowest speed.
    ///
    /// # Arguments
    ///
    /// * `name` - Driver's name
    /// * `ai` - AI::Human or AI::NPC
    pub fn new(name: String, ai: AI) -> Racer {
        Racer {
            name: name,
            hp: 0,
            position: Position { lap: 0, turn: 0 },
            speed: -2,
            alive: true,
            ai: ai,
            turn: 0,
            log: vec!(Vec::new())
        }
    }

    /// Return first letter of racer name.
    pub fn shortname(&self) -> String {
        let mut name = self.name.clone();
        while name.len() > 1 {
            name.pop();
        }
        name
    }

    /// Set HP to default value.
    ///
    /// The default HP value is two more than the highest turn ranking of the circuit
    ///
    /// # Arguments
    /// * `turns` - the circuit
    pub fn reset_hp(&mut self, turns: &Vec<isize>) {
        self.hp = turns.iter().max().unwrap() + 2;
    }

    /// Increment speed within the interval [-2,3].
    pub fn speed_up(&mut self) {
        if self.speed < 3 {
            self.log[self.turn].push(format!("{} speeds up to '{}' speed!",
                     self.name,
                     speed_adj(&(self.speed + 1))));
            self.speed += 1
        }
    }

    /// Decrement speed within the interval [-2,3].
    pub fn slow_down(&mut self) {
        if self.speed > -2 {
            self.log[self.turn].push(format!("{} slows down to '{}' speed!",
                     self.name,
                     speed_adj(&(self.speed - 1))));
            self.speed -= 1
        }
    }

    /// Add speed modifier to the next turn ranking in the circuit.
    ///
    /// # Arguments
    ///
    /// * `turns` - the circuit
    pub fn eval_difficulty(&self, turns: &Vec<isize>) -> isize {
        self.speed + self.next_turn(turns)
    }

    /// Reduce player's HP.
    ///
    /// # Arguments
    ///
    /// * `value` - points of damage to take
    pub fn take_damage(&mut self, value: &isize) {
        self.hp -= value.clone();
        self.log[self.turn].push(format!("\u{1b}[31;1m{}'s car takes {} point{} of damage!\u{1b}[0m",
                 self.name,
                 value,
                 {if *value == 1 {""} else {"s"}}));
        // Spin out!
        if self.hp <= 0 {
            self.log[self.turn].push(format!("\u{1b}[31;4;1m{}'s car is totalled! {} is out of the race!\u{1b}[0m",
                     self.name, self.name));
            self.alive = false;
            self.hp = 0;
        }
    }

    pub fn get_log(&mut self) -> Vec<String> {
        println!("{} - {}", self.log.len(), self.turn);
        if self.log.len() <= self.turn {
            vec!(String::new())
        } else {
            self.turn += 1;
            self.log.push(Vec::new());
            self.log[self.turn - 1].clone()
        }
    }

    /// Advance along the circuit.
    ///
    /// Long Description.
    ///
    /// # Arguments
    ///
    /// * `steps` - number of steps to move
    /// * `size` - length of one lap
    pub fn move_steps(&mut self, steps: usize, size: usize) {
        self.position.turn += steps;
        // Pass lap end
        if self.position.turn >= size {
            self.position.lap += 1;
            self.log[self.turn].push(format!("\u{1b}[4;1m{} crosses the start/finish line, entering lap {}!\u{1b}[0m",
                     self.name,
                     self.position.lap + 1));
            self.position.turn = 0;
        }
    }

    /// Return the number of steps the racer will take at the current speed.
    ///
    /// If the speed ranking is:
    /// - negative, 1 step
    /// - 0, 2 steps
    /// - positive, 3 steps
    pub fn move_range(&self) -> isize {
        if self.speed < 0 {
            1
        } else if self.speed == 0 {
            2
        } else {
            3
        }
    }

    /// Return the next turn ranking.
    ///
    /// Find the highest turn ranking in the racer's move range.
    ///
    /// # Arguments
    ///
    /// * `turns` - the circuit
    pub fn next_turn(&self, turns: &Vec<isize>) -> isize {
        let mut ups = Vec::new();
        for i in 0..self.move_range() {
            let item = self.position.turn + i as usize;
            if item < turns.len() {
                ups.push(turns[item]);
            } else {
                ups.push(turns[item - turns.len()]);
            }
        }
        ups.iter().max().unwrap().clone()
    }

    /// Attempt to make the next turn.
    ///
    /// See instructions for details.
    ///
    /// # Arguments
    ///
    /// * `turns` - the circuit
    pub fn make_turn(&mut self, turns: &Vec<isize>) {
        // The value needed to succeed
        let need = self.eval_difficulty(turns);
        // The racer's roll
        let roll = roll();
        let diff = roll as isize - need;
        let step = self.move_range() as usize;

        if diff >= 0 {
            // Success!
            let msg = format!("\u{1b}[32;1m{} makes the {} turn!\u{1b}[0m",
                              self.name,
                              turn_adj(&self.next_turn(turns)));
            self.log[self.turn].push(msg);
            self.move_steps(step, turns.len());
        } else {
            // Fail!
            let msg = format!("\u{1b}[31;1m{} fails the {} turn!\u{1b}[0m",
                              self.name,
                              turn_adj(&self.next_turn(turns)));
            self.log[self.turn].push(msg);
            // Take damage
            self.take_damage(&(0 - diff));
            // Move one less step
            if self.alive {
                self.move_steps(step - 1, turns.len());
            }
        }
    }

    // This doesn't work. Can't access racers during main loop
    /*
    pub fn try_bump(&mut self, players: &[Racer]) {
        let mut hittable = players.to_vec();
        let mut hittable = hittable.iter_mut()
            .filter(|x| x.name != self.name)
            .filter(|y| {
                (y.position.turn as isize - self.position.turn as isize).abs() <= self.move_range()
            }).collect::<Vec<&mut Racer>>();
        let mut rng = thread_rng();
        rng.shuffle(&mut hittable);
        if roll() >= 4 {
            let damage = roll() as isize + self.speed;
            match roll() {
                1 => self.take_damage(&damage),
                6 => {self.take_damage(&damage); hittable[0].take_damage(&damage);},
                _ => hittable[0].take_damage(&damage)
            }
        }
    }

    pub fn ai_move(&mut self, turns: &Vec<isize>, racers: &[Racer]) {
        let can_heal = |player: &Racer| player.move_range() as usize + player.position.turn >= turns.len();
        let basic_move = |player: &mut Racer|
            if can_heal(player) {
                player.reset_hp(&turns);
            } else {
                match player.next_turn(&turns) {
                    1 | 2 => player.speed_up(),
                    _     => player.slow_down()
                }
                player.make_turn(turns);
            };


        let ai = self.ai.clone();
        match ai {
            AI::Human => (),
            AI::NPC(brain)        => match brain {
                Brain::Slug       => self.speed = -1,
                Brain::Nocombat   => basic_move(self),
                Brain::Aggressive => if roll() >= 4 {
                    self.try_bump(racers); basic_move(self)},
                Brain::Beast      => {self.speed = 1; self.try_bump(racers)},
                Brain::Deathwish  => self.speed = 3,
                Brain::Lurker     => {
                    self.speed = -2;
                    if can_heal(self) {
                        self.reset_hp(&turns);
                    } else {
                        self.try_bump(racers);
                    }}
            }
        }
    }
    */

}

/// Get the label for each speed ranking.
///
/// # Arguments
///
/// * `speed` - the speed ranking
pub fn speed_adj(speed: &isize) -> &str {
    match *speed {
        -2 => "crawling",
        -1 => "edging",
        0  => "cruising",
        1  => "speeding",
        2  => "racing",
        3  => "tempting death",
        _  => "freaking out"
    }
}

/// Get the label for each turn ranking.
///
/// # Arguments
///
/// * `turn` - the turn ranking
pub fn turn_adj(turn: &isize) -> &str {
    match *turn {
        1 => "straight",
        2 => "slight",
        3 => "smooth",
        4 => "sharp",
        5 => "banked",
        6 => "U",
        _ => "impossible"
    }
}
