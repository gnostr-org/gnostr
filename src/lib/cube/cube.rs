use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use std::fmt::{self, Formatter};

#[derive(Eq, PartialEq, Copy, Clone)]
enum Face {
    U,
    L,
    F,
    R,
    B,
    D,
    None,
}

impl std::fmt::Display for Face {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let c = match self {
            Face::U => 'U',
            Face::L => 'L',
            Face::F => 'F',
            Face::R => 'R',
            Face::B => 'B',
            Face::D => 'D',
            Face::None => unreachable!(),
        };
        write!(f, "{}", c)?;
        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Move {
    face: Face,
    prime: bool,
    half_turn: bool,
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let primestr = match self.prime {
            true => "\'",
            false => "",
        };
        let halfstr = match self.half_turn {
            true => "2",
            false => "",
        };
        write!(f, "{}{}{}", self.face, halfstr, primestr)?;
        Ok(())
    }
}

impl Move {
    fn new(last: Move) -> Move {
        let next: Move = rand::random();
        if next.face == last.face {
            Move::new(last)
        } else {
            next
        }
    }
}

impl Distribution<Move> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Move {
        let face = match rng.gen_range(0..6) {
            0 => Face::U,
            1 => Face::L,
            2 => Face::F,
            3 => Face::R,
            4 => Face::B,
            5 => Face::D,
            _ => unreachable!(),
        };
        let (prime, half_turn) = match rng.gen_range(0..3) {
            0 => (false, false),
            1 => (true, false),
            2 => (false, true),
            _ => unreachable!(),
        };
        Move {
            face,
            prime,
            half_turn,
        }
    }
}

pub fn gen_scramble() -> String {
    let mut s = String::new();
    let mut l = Move {
        face: Face::None,
        prime: false,
        half_turn: false,
    };
    let _ = (0..21)
        .into_iter()
        .map(|_| {
            l = Move::new(l);
            s += &(" ".to_owned() + &l.to_string())
        })
        .collect::<()>();
    s
}
