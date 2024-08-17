use anyhow::Result;
use colored::Colorize;
use std::{cmp::Ordering, io};
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

#[derive(Debug, Clone, Copy)]
enum PlayerChoice {
    PlayCard,
    Quit,
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
            player: VecDeque::from(deck.cards[0..=25].to_vec()),
            computer: VecDeque::from(deck.cards[26..].to_vec()),
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
    fn get_input(&self) -> Result<PlayerChoice> {
        println!("Enter 1 to play a card or 2 to quit");
        loop {
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().parse::<u8>()?;
            match choice {
                1 => {
                    return Ok(PlayerChoice::PlayCard);
                },
                2 => {
                    return Ok(PlayerChoice::Quit);
                },
                _ => {
                    eprintln!("please try again");
                    continue
                }
            }
        }
    }
    fn play_round(&mut self) -> bool {
        if self.player.is_empty() {
            println!("oh no, you lose :/\n");
            return false
        } 
        if self.computer.is_empty() {
            println!("you win!!!\n");
            return false
        }
        match self.get_input().expect("player should have made a choice") {
            PlayerChoice::Quit => {
                println!("thanks for playing!\n");
                return false;
            }
            PlayerChoice::PlayCard => {},
        }
        let player_card = self.player.pop_front().expect("player should have a card for round");
        let computer_card = self.computer.pop_front().expect("computer should have a card for round");
        let mut winning_cards = Vec::new();
        match player_card.rank.to_u8().cmp(&computer_card.rank.to_u8()) {
            Ordering::Greater => {
                println!("you win, {} beats {}\n", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.player.extend(winning_cards);
                true
            },
            Ordering::Less => {
                println!("you lose, {} gets beat by {}\n", player_card, computer_card);
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.computer.extend(winning_cards);
                true
            },
            Ordering::Equal => {
                println!("uh oh, time for war!\n");
                winning_cards.push(player_card);
                winning_cards.push(computer_card);
                self.resolve_war(winning_cards);
                true
            }
        }
    }
    fn resolve_war(&mut self, mut winning_cards: Vec<Card>) {
        let mut last_card = false;
        let player_card = match self.player.len() {
            0 => {
                last_card = true;
                winning_cards[0]
            },
            1 => self.player.pop_front().expect("player's last card"),
            _ => {
                let discard = self.player.pop_front().expect("player should have spare cards");
                winning_cards.push(discard);
                self.player.pop_front().expect("player's card")
            },
        };
        let computer_card = match self.computer.len() {
            0 => {
                last_card = true;
                winning_cards[1]
            },
            1 => self.computer.pop_front().expect("computer's last card"),
            _ => {
                let discard = self.computer.pop_front().expect("computer should have spare cards");
                winning_cards.push(discard);
                self.computer.pop_front().expect("computer's card")
            },
        };
        match player_card.rank.to_u8().cmp(&computer_card.rank.to_u8()) {
            Ordering::Greater => {
                println!("you win the war, {} beats {}\n", player_card, computer_card);
                // card is already in winning_cards (was passed into this function)
                if !last_card {
                    winning_cards.push(player_card);
                }
                winning_cards.push(computer_card);
                self.player.extend(winning_cards);
            },
            Ordering::Less => {
                println!("you lose the war, {} gets beat by {}\n", player_card, computer_card);
                winning_cards.push(player_card);
                // card is already in winning_cards (was passed into this function)
                if !last_card {
                    winning_cards.push(computer_card);
                }
                self.computer.extend(winning_cards);
            },
            Ordering::Equal => {
                println!("time for another war!\n");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::Suit;

    #[test]
    fn test_new_game_initialization() {
        let war = War::new();
        assert_eq!(war.player.len(), 26);
        assert_eq!(war.computer.len(), 26);
    }
    #[test]
    fn test_play_round_player_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Clubs, rank: Rank::Four },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..2].to_vec()),
            computer: VecDeque::from(deck.cards[2..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 3);
        assert_eq!(war.computer.len(), 1);
    }
    #[test]
    fn test_play_round_computer_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Four },
            Card { suit: Suit::Clubs, rank: Rank::Five },
            Card { suit: Suit::Clubs, rank: Rank::Six },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..2].to_vec()),
            computer: VecDeque::from(deck.cards[2..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 1);
        assert_eq!(war.computer.len(), 3);
    }
    #[test]
    fn test_play_round_war() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Eight },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Eight },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..=2].to_vec()),
            computer: VecDeque::from(deck.cards[3..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 6);
        assert_eq!(war.computer.len(), 0);
    }
    #[test]
    fn test_play_round_war_no_cards_remaining_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            player: VecDeque::from(vec![deck.cards[0]]),
            computer: VecDeque::from(deck.cards[1..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 4);
        assert_eq!(war.computer.len(), 0);
    }
    #[test]
    fn test_play_round_war_no_cards_remaining_lose() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            // swap from last test
            computer: VecDeque::from(vec![deck.cards[0]]),
            player: VecDeque::from(deck.cards[1..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 0);
        assert_eq!(war.computer.len(), 4);
    }
    #[test]
    fn test_play_round_war_one_card_remaining_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            // make sure this is skipped correctly
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..=1].to_vec()),
            computer: VecDeque::from(deck.cards[2..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 5);
        assert_eq!(war.computer.len(), 0);
    }
    #[test]
    fn test_play_round_war_one_card_remaining_lose() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            computer: VecDeque::from(deck.cards[0..=1].to_vec()),
            player: VecDeque::from(deck.cards[2..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 0);
        assert_eq!(war.computer.len(), 5);
    }
    #[test]
    fn test_play_round_war_two_cards_remaining_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..=2].to_vec()),
            computer: VecDeque::from(deck.cards[3..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 6);
        assert_eq!(war.computer.len(), 0);
    }
    #[test]
    fn test_play_round_war_two_cards_remaining_lose() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Five },
        ];
        let mut war = War {
            computer: VecDeque::from(deck.cards[0..=2].to_vec()),
            player: VecDeque::from(deck.cards[3..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 0);
        assert_eq!(war.computer.len(), 6);
    }
    #[test]
    fn test_play_round_double_win() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Four },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Two },
            Card { suit: Suit::Hearts, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
        ];
        let mut war = War {
            player: VecDeque::from(deck.cards[0..=4].to_vec()),
            computer: VecDeque::from(deck.cards[5..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 10);
        assert_eq!(war.computer.len(), 1);
    }
    #[test]
    fn test_play_round_double_war_lose() {
        let mut deck = Deck::new();
        deck.cards = vec![
            Card { suit: Suit::Hearts, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Six },
            Card { suit: Suit::Hearts, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Four },
            Card { suit: Suit::Hearts, rank: Rank::King },
            Card { suit: Suit::Hearts, rank: Rank::Jack },
            Card { suit: Suit::Spades, rank: Rank::King },
            Card { suit: Suit::Diamonds, rank: Rank::King },
            Card { suit: Suit::Clubs, rank: Rank::Seven },
            Card { suit: Suit::Hearts, rank: Rank::Five },
            Card { suit: Suit::Clubs, rank: Rank::Nine },
            Card { suit: Suit::Hearts, rank: Rank::Two },
            Card { suit: Suit::Hearts, rank: Rank::Three },
            Card { suit: Suit::Hearts, rank: Rank::Queen },
            Card { suit: Suit::Diamonds, rank: Rank::Queen },
        ];
        let mut war = War {
            computer: VecDeque::from(deck.cards[0..=7].to_vec()),
            player: VecDeque::from(deck.cards[8..].to_vec()),
        };
        war.play_round();
        assert_eq!(war.player.len(), 2);
        assert_eq!(war.computer.len(), 13);
    }
}
