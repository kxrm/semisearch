# SemiSearch for Beginners

## What is SemiSearch?

SemiSearch helps you find text in your files, even when you don't remember the exact words.

## Basic Usage

Just type what you're looking for:
```bash
semisearch "what you want to find"
```

That's it! No configuration required.

## Examples

### Find TODO comments
```bash
semisearch "TODO"
```

### Find database-related code
```bash
semisearch "database"
```

### Find error handling
```bash
semisearch "error"
```

### Find function definitions
```bash
semisearch "function login"
```

## When Things Go Wrong

### No results found?
Try:
- Check spelling: `semisearch "your search" --fuzzy`
- Use simpler words: `semisearch "login"` instead of `semisearch "authentication"`
- Search everywhere: `semisearch "your search" .`

### Too many results?
Try:
- Be more specific: `semisearch "function validateUser"`
- Search in specific folders: `semisearch "TODO" src/`

### Still stuck?
Run: `semisearch help-me` for interactive help.

## Interactive Help

SemiSearch includes an interactive help system that guides you through common scenarios:

```bash
semisearch help-me
```

This will start an interactive session where you can:
- Type search queries and see examples
- Get personalized suggestions
- Learn about different search options
- Practice with real examples

### Interactive Commands

Within the interactive help, you can use:
- `help` or `?` - Show available commands
- `examples` - Show common search examples
- `tips` - Show search tips and best practices
- `quit` or `exit` - Exit interactive help

## Search Flags

### Handle typos and similar words
```bash
semisearch "databse" --fuzzy
```
This will find "database" even with the typo!

### Find exact matches only
```bash
semisearch "exact phrase" --exact
```

### Show more results
```bash
semisearch "TODO" --limit 20
```

### Search in specific locations
```bash
semisearch "function" src/          # Search only in src/ directory
semisearch "config" config.json     # Search in specific file
```

## Getting Help

### Check if everything is working
```bash
semisearch status
```

### Interactive help and tutorials
```bash
semisearch help-me
```

### Quick command reference
```bash
semisearch --help
```

### Advanced options (for power users)
```bash
semisearch --advanced --help
```

## Common Use Cases

### For Developers
- `semisearch "TODO"` - Find all TODO comments
- `semisearch "async function"` - Find async functions
- `semisearch "import React"` - Find React imports
- `semisearch "try catch"` - Find error handling

### For Writers
- `semisearch "needs revision"` - Find draft sections
- `semisearch "citation needed"` - Find unsourced claims
- `semisearch "methodology"` - Find research methods

### For Configuration
- `semisearch "password"` - Find password-related items
- `semisearch "config"` - Find configuration files
- `semisearch "port 8080"` - Find port configurations

## Tips for Better Results

### Start Simple, Then Get Specific
1. Start with: `semisearch "login"`
2. If too many results: `semisearch "login function"`
3. If still too many: `semisearch "function validateLogin"`

### Use Different Words
If "error" doesn't work, try:
- `semisearch "exception"`
- `semisearch "fail"`
- `semisearch "catch"`

### Search in the Right Place
- Code: `semisearch "function" src/`
- Documentation: `semisearch "tutorial" docs/`
- Configuration: `semisearch "port" config/`

### Handle Typos
Always add `--fuzzy` when you're not sure about spelling:
```bash
semisearch "databse connection" --fuzzy
```

## What Makes SemiSearch Special?

### Smart Search
SemiSearch automatically chooses the best search method based on what you're looking for:
- Simple words → Fast keyword search
- Complex phrases → Intelligent matching
- Code patterns → Code-aware search

### Helpful Errors
When searches don't work, SemiSearch tells you exactly what to try next, with specific examples.

### No Setup Required
Just install and start searching. SemiSearch works out of the box.

## Next Steps

Once you're comfortable with basic searching:

1. **Learn about indexing**: `semisearch index .` to make searches faster
2. **Explore advanced features**: `semisearch --advanced --help`
3. **Check system status**: `semisearch status` to see all capabilities
4. **Run diagnostics**: `semisearch doctor` for detailed system information

## Need More Help?

- **Interactive help**: `semisearch help-me`
- **Quick status check**: `semisearch status`
- **Full documentation**: Check the other files in the `docs/` folder
- **Advanced features**: `semisearch --advanced --help`

Remember: SemiSearch is designed to be helpful. If you're stuck, the tool will guide you to the solution! 