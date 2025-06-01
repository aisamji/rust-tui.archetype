let context = #{};

// This generally comes from ANSWERS, but prompting for it ensures it is in the context 
context += prompt("Project Author:", "author_full", #{
    placeholder: "Your Name <your.name@example.com>",
});

context += prompt("Project Name:", "project-name", #{
  cases: [
    CasedValue(KebabCase), 
  ], 
});

context += prompt("Short description:", "description", #{
  defaults_with: "A TUI application written in Rust.",
});

context += prompt("Github owner:", "github-owner");

render(Directory("contents"), context)
