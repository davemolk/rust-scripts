use math::User;

fn main() {
    let mut user = User::new();
    if let Err(e) = user.play() {
        eprintln!("{}", e);
        std::process::exit(1)
    }
}
