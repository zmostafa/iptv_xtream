slint::include_modules!();

struct App {
    main_view: MainView,
}

impl App {
    fn new() -> Self {
        let main_view = MainView::new().unwrap();

        // Start with the login screen (page 6)
        // main_view.set_active_page(6).unwrap();

        Self { main_view }
    }

    fn run(&self) {
        let main_view_weak = self.main_view.as_weak();

        // Handle login
        self.main_view.on_login(move |url, username, password| {
            let main_view = main_view_weak.unwrap();

            // Simulate login validation
            if !username.is_empty() && !password.is_empty() {
                println!("Login successful!");

                // Switch to the home page (page 0) after successful login
                main_view.set_active_page(0);
            } else {
                println!("Login failed: Username and password cannot be empty.");
            }
        });

        // Run the MainView
        self.main_view.run().unwrap();
    }
}

fn main() {
    let app = App::new();
    app.run();
}