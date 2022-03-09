use std::{collections::{HashMap, HashSet}, fmt::{Display, Formatter}};
use actix_web::HttpRequest;
use uuid::Uuid;

use crate::words::get_random_word;

/*
 * Some utility functions
 */
pub fn get_player_uuid(req: &HttpRequest) -> Option<Uuid> 
{
    if let Some(cookie) = req.cookie("userid") {
        let uuid_val = Uuid::parse_str(cookie.value());
        if let Ok(uuid) = uuid_val {
            return Some(uuid);
        }
    }
    return None;
}

/*
 * Helper structs
 */
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
pub enum CharColor
{
    Yellow,
    Green,
    Gray
}

impl Display for CharColor
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error>
    {
        match self
        {
            &Self::Yellow => f.write_str("Y")?,
            &Self::Green => f.write_str("G")?,
            &Self::Gray => f.write_str("Gr")?
        };
        Ok(())
    }
}

#[derive(Default)]
pub struct WordGuess 
{
    word: String,
    char_states: Vec<CharColor>
}

impl Display for WordGuess
{
    fn fmt(&self, a: &mut Formatter) -> Result<(), std::fmt::Error> {
        for (cc, c) in self.char_states.iter().zip(self.word.clone().chars()) { 
            a.write_str(&format!("{}:{}", cc, c))?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct PlayerState
{
    // player's display name
    pub name: String,
    // whether a player is an owner of their lobby
    pub is_owner: bool,
    // Some(lobby_id) or None
    pub lobby: Option<String>,
    // A list of the player's guesses so far.
    pub word_guesses: Vec<WordGuess>
}

impl PlayerState {

    pub fn has_won(&self) -> bool
    {
        self.word_guesses
            .iter()
            .map(|a| a.char_states.iter())
            .all(|mut b| 
                 b.all(|a| *a == CharColor::Green))
    }

}

#[derive(Default)]
pub struct LobbyState
{
    pub started: bool,
    pub ended: bool,
    pub chosen_word: String,
    pub players: Vec<Uuid>
}

/*
 * The server object which contains connection infos
 */
#[derive(Default)]
pub struct Server 
{
    //Player connections by Cookie ID
    connections: HashMap<Uuid, PlayerState>,
    lobbies: HashMap<String, LobbyState>
}

impl Server {
    pub fn new() -> Self {
        Server {
            connections: HashMap::new(),
            lobbies: HashMap::new()
        }
    }

    #[allow(dead_code)]
    pub fn create_lobby(&mut self, lobby_name: String)
    {
        self.lobbies.insert(lobby_name, LobbyState {
            chosen_word: get_random_word().into(),
            ..LobbyState::default()
        });
    }

    pub fn get_player(&self, uuid: &Uuid) -> &PlayerState
    {
        return self.connections.get(uuid).unwrap();
    }

    pub fn get_player_mut(&mut self, uuid: &Uuid) -> &mut PlayerState {
        return self.connections.get_mut(uuid).unwrap();
    }

    pub fn does_lobby_exist(&self, s: &String) -> bool {
        self.lobbies.contains_key(s)
    }

    // Player assumed to be valid lobby holder && guess is correct len
    pub fn player_submit_move(&self, player: &Uuid, guess: String) -> WordGuess
    {
        let player = self.get_player(player);
        let lobby_ref = self.lobbies.get(player.lobby.as_ref().unwrap()).unwrap();
        let lobby_word = lobby_ref.chosen_word.clone();
        let mut lobby_chars = lobby_word.chars();
        let lobby_set: HashSet<char> = lobby_chars.clone().collect();
        let mut built_guess = WordGuess {
            word: guess.clone(),
            char_states: vec![]
        };
        for c_guess in guess.chars() {
            let rel_char = lobby_chars.next().expect("Guess / Lobby word not same size");
            if c_guess == rel_char {
                built_guess.char_states.push(CharColor::Green);
            } else if lobby_set.contains(&c_guess) {
                built_guess.char_states.push(CharColor::Yellow);
            } else {
                built_guess.char_states.push(CharColor::Gray);
            }
        }
        return built_guess;
    }

    pub fn is_player_valid(&self, s: &Uuid) -> bool {
        self.connections.contains_key(s)
    }

    pub fn init_player(&mut self, s: &Uuid) {
        self.connections.insert(s.clone(), PlayerState::default());
    }

    pub fn has_lobby(&self, s: &Uuid) -> bool {
        let player_ref = self.connections.get(&s).expect("Player doesn't exist");
        player_ref.lobby.is_some() && self.lobbies.contains_key(player_ref.lobby.as_ref().unwrap())
    }

    pub fn any_winners(&self, lobby: &String) -> Option<Uuid> {
        let vec_players = self.lobbies.get(lobby).expect("Called any_winners without verifying existence").players.iter();
        vec_players.filter(|a| self.get_player(&a).has_won()).next().map(|a| a.clone())
    }

    pub fn end_game(&mut self, lobby: &String) {
        let lobby_players = self.lobbies.get(lobby).unwrap().players.clone();
        for player in lobby_players {
            self.get_player_mut(&player).lobby = None
        }
        self.lobbies.remove(lobby);
    }
}

/*
 *
 * Tests
 *
 */

#[cfg(test)]
mod tests {
    use crate::server;

    #[test]
    fn test_server_guess()
    {
        let mut server = server::Server::default();
        let uuid = uuid::Uuid::new_v4();
        // Add player to server connection with id UUID
        server.init_player(&uuid);
        // Inits lobby w/ random word
        server.create_lobby("test".into());
        // Gets a reference to playerstate
        server.get_player_mut(&uuid).lobby = Some("test".into());
        // Submits a move for the player by id and string guess
        println!("{}\n", server.player_submit_move(&uuid, "zzzzz".into()));
    }
}
