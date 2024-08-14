use anyhow::Result;
use colored::Colorize;
use std::cmp::Ordering;
use std::collections::VecDeque;

use super::{
    color,
    deck::{Card, Deck, Rank},
    ascii,
};

#[derive(Debug)]
struct War {
    player: VecDeque<Card>,
    computer: VecDeque<Card>,
}

impl Rank {
    pub fn to_u8(self) -> u8 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 11,
            Rank::Queen => 12,
            Rank::King => 13,
            Rank::Ace => 14,
        }
    }
}

impl War {
    fn new() -> Self {
        let mut deck = Deck::new();
        deck.shuffle();
        War{
            player: deck.cards[0..=25].to_vec().into(),
            computer: deck.cards[26..].to_vec().into(),
        }
    }
    fn run_game_loop(&mut self) {
        loop {
            let keep_playing = self.play_round();
            if !keep_playing {
                break
            }
        }
    }
    fn play_round(&mut self) -> bool {
        if self.player.is_empty() {
            println!("oh no, you lose :/");
            return false
        } 
        if self.computer.is_empty() {
            println!("you win!!!");
            return false
        }

        println!("player has {}", self.player.len());
        println!("computer has {}", self.computer.len());

        let player_card = self.player.pop_front().expect("player should have a card for round");
        let computer_card = self.computer.pop_front().expect("computer should have a card for round");
        let mut winning_cards = Vec::new();
        match player_card.rank.to_u8().cmp(&computer_card.rank.to_u8()) {
            Ordering::Greater => {
                println!("you win, {} beats {}", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.player.extend(winning_cards);
                true
            },
            Ordering::Less => {
                println!("you lose, {} gets beat by {}", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.computer.extend(winning_cards);
                true
            },
            Ordering::Equal => {
                println!("uh oh, time for war!");
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.resolve_war(winning_cards);
                true
            }
        }
    }
    fn resolve_war(&mut self, mut winning_cards: Vec<Card>) {
        let player_card = match self.player.len() {
            0 => winning_cards[0],
            1 => self.player.pop_front().expect("player's last card"),
            _ => {
                let discard = self.player.pop_front().expect("player should have spare cards");
                winning_cards.push(discard);
                self.player.pop_front().expect("player's card")
            },
        };
        let computer_card = match self.computer.len() {
            0 => winning_cards[1],
            1 => self.computer.pop_front().expect("computer's last card"),
            _ => {
                let discard = self.computer.pop_front().expect("computer should have spare cards");
                winning_cards.push(discard);
                self.computer.pop_front().expect("computer's card")
            },
        };
        match player_card.rank.to_u8().cmp(&computer_card.rank.to_u8()) {
            Ordering::Greater => {
                println!("you win the war, {} beats {}", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.player.extend(winning_cards);
            },
            Ordering::Less => {
                println!("you lose the war, {} gets beat by {}", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.computer.extend(winning_cards);
            },
            Ordering::Equal => {
                println!("time for another war!");
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.resolve_war(winning_cards);
            }
        }
    }
}

pub fn run_war() -> Result<()> {
    let mut war = War::new();
    let (r, g, b) = color();
    println!("{}", ascii::WAR.truecolor(r, g, b));
    war.run_game_loop();
    Ok(())
}