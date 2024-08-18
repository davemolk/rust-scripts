use rand::seq::SliceRandom;
use rand::thread_rng;
use core::fmt;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Hearts,
    Spades,
    Clubs,
    Diamonds,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

#[derive(Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl fmt::Debug for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} of {:?}",
            self.rank,
            self.suit
        )
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank = match self.rank {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        };
        let suit = match self.suit {
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
        };
        write!(
            f,
            "┌─────┐\n\
             │{}{}   │\n\
             │  {}  │\n\
             │  {}{} │\n\
             └─────┘",
            if rank.len() == 2 {""} else {" "},
            rank,
            suit,
            if rank.len() == 2 {""} else {" "},
            rank
        )
    }
}

impl Card {
    pub fn emoji(&self) -> String {
        let rank = match self.rank {
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
            Rank::Ace => "A",
        };

        let suit = match self.suit {
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
        };

        format!("{rank} of {suit}")
    }
}


impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        for &suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &rank in &[
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack,
                Rank::Queen, Rank::King, Rank::Ace,
            ] {
                cards.push(Card{suit, rank});
            }
        }
        Deck{cards}
    }
    pub fn shuffle(&mut self) {
        let mut rng = thread_rng();
        self.cards.shuffle(&mut rng);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new() {
        let deck = Deck::new();
        assert_eq!(52, deck.cards.len());
        // 52 unique cards
        let mut set = std::collections::HashSet::new();
        for card in &deck.cards {
            set.insert((card.suit, card.rank));
        }
        assert_eq!(52, set.len());
        // all expected are included
        for &suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
            for &rank in &[
                Rank::Two, Rank::Three, Rank::Four, Rank::Five, Rank::Six,
                Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, Rank::Jack,
                Rank::Queen, Rank::King, Rank::Ace,
            ] {
                assert!(set.contains(&(suit, rank)))
            }
        }
    }
    #[test]
    fn shuffle() {
        let mut deck1 = Deck::new();
        let mut deck2 = deck1.clone();
        deck2.shuffle();
        assert_eq!(52, deck2.cards.len());
        let same_order = deck1.cards.iter().zip(deck2.cards.iter()).all(|(a, b)| a == b);
        assert!(!same_order, "order didn't change with shuffling");
        deck1.cards.sort_by(|a, b| a.cmp(b));
        deck2.cards.sort_by(|a, b| a.cmp(b));
        assert_eq!(deck1.cards, deck2.cards);
    }
}
