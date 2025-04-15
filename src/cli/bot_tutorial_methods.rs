    /// Start a tutorial
    pub async fn start_tutorial(&mut self, tutorial_id: &str) -> Result<()> {
        // Check if tutorial manager is available
        let tutorial_manager = match &self.tutorial_manager {
            Some(manager) => manager,
            None => return Err(anyhow!("Tutorial manager not available")),
        };
        
        // Get the tutorial
        let tutorial = match tutorial_manager.get_tutorial(tutorial_id) {
            Some(tutorial) => tutorial.clone(),
            None => return Err(anyhow!("Tutorial not found: {}", tutorial_id)),
        };
        
        // Create a new tutorial session
        let session = crate::bot::tutorial::TutorialSession::new(tutorial);
        self.active_tutorial = Some(session);
        
        // Show the first step
        self.show_current_tutorial_step()
    }
    
    /// Show the current tutorial step
    pub fn show_current_tutorial_step(&self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };
        
        // Format and print the current step
        let step_text = session.format_current_step();
        println!("{}: {}\n", branding::colorize("Tutorial", branding::Color::Cyan), step_text);
        
        Ok(())
    }
    
    /// Move to the next tutorial step
    pub fn next_tutorial_step(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &mut self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };
        
        // Move to the next step
        if session.next_step().is_none() {
            // Tutorial completed
            println!("{}: {}\n", 
                branding::colorize("Tutorial", branding::Color::Cyan), 
                "Congratulations! You've completed the tutorial.");
            
            // Clear the active tutorial
            self.active_tutorial = None;
            
            return Ok(());
        }
        
        // Show the current step
        self.show_current_tutorial_step()
    }
    
    /// Move to the previous tutorial step
    pub fn previous_tutorial_step(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        let session = match &mut self.active_tutorial {
            Some(session) => session,
            None => return Err(anyhow!("No active tutorial")),
        };
        
        // Move to the previous step
        session.previous_step();
        
        // Show the current step
        self.show_current_tutorial_step()
    }
    
    /// Exit the current tutorial
    pub fn exit_tutorial(&mut self) -> Result<()> {
        // Check if there's an active tutorial
        if self.active_tutorial.is_none() {
            return Err(anyhow!("No active tutorial"));
        }
        
        // Clear the active tutorial
        self.active_tutorial = None;
        
        println!("{}: {}\n", 
            branding::colorize("Tutorial", branding::Color::Cyan), 
            "Tutorial exited. You can start another tutorial by typing !tutorial");
        
        Ok(())
    }
    
    /// List available tutorials
    pub fn list_tutorials(&self) -> Result<String> {
        // Check if tutorial manager is available
        let tutorial_manager = match &self.tutorial_manager {
            Some(manager) => manager,
            None => return Err(anyhow!("Tutorial manager not available")),
        };
        
        // Get all tutorials
        let tutorials = tutorial_manager.get_all_tutorials();
        
        if tutorials.is_empty() {
            return Ok("No tutorials available.".to_string());
        }
        
        // Format the tutorial list
        let mut result = String::new();
        result.push_str("Available Tutorials:\n\n");
        result.push_str(&tutorial_manager.format_tutorial_list(tutorials));
        result.push_str("\nTo start a tutorial, type !tutorial <id>\n");
        
        Ok(result)
    }
