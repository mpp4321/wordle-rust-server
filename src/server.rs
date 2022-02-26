use std::{collections::{HashMap, HashSet}, fmt::{Display, Formatter}};
use actix_web::HttpRequest;
use uuid::Uuid;

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
#[derive(Debug)]
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
    // whether a player is an owner of their lobby
    pub is_owner: bool,
    // Some(lobby_id) or None
    pub lobby: Option<String>,
    // A list of the player's guesses so far.
    pub word_guesses: Vec<WordGuess>
}

#[derive(Default)]
pub struct LobbyState
{
    pub started: bool,
    pub ended: bool,
    pub chosen_word: String,
    pub players: Vec<u32>
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

    pub fn create_lobby(&mut self, lobby_name: String)
    {
        self.lobbies.insert(lobby_name, LobbyState::default());
    }

    pub fn set_word(&mut self, lobby: &String, word: String)
    {
        self.lobbies
            .get_mut(lobby)
            .unwrap()
            .chosen_word = word;
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
            word: lobby_word.clone(),
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
        player_ref.lobby.is_some() && self.lobbies.contains_key(&player_ref.lobby.unwrap())
    }

    pub fn update_wins(&mut self)
    {
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
        server.init_player(&uuid);
        server.create_lobby("test".into());
        server.get_player_mut(&uuid).lobby = Some("test".into());
        server.set_word(&"test".into(), "testa".into());
        println!("{}", format!("{}", server.player_submit_move(&uuid, "testb".into())) );
    }
}
