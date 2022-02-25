#[cfg(test)]
mod tests {
    use server;
    #[test]
    fn test_server_guess()
    {
        let mut server = server::Server::default();
        let uuid = uuid::Uuid::new_v4();
        server.init_player(&uuid);
        let player_ref = server.get_player(&uuid);
        server.create_lobby("test".into());
        server.set_word("testa");
        print!("{?:}", server.player_submit_move(player_ref, "testb"));
    }
}

