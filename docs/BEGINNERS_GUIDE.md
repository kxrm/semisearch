# SemiSearch for Beginners

## What is SemiSearch?

SemiSearch helps you find text in your files, even when you don't remember the exact words. It's designed to be simple for beginners but grows with you as you learn.

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

### Search in specific places
```bash
semisearch "TODO" src/          # Search only in src/ directory
semisearch "config" config.json # Search in specific file
```

## When Things Go Wrong

### No results found?
SemiSearch will automatically suggest what to try:
```bash
semisearch "nonexistent"
# Shows helpful suggestions like:
# • Try different words
# • Check spelling with --fuzzy
# • Search in parent directories
```

Try:
- **Check spelling**: `semisearch "databse" --fuzzy` (finds "database")
- **Use simpler words**: `semisearch "login"` instead of `semisearch "authentication"`
- **Search everywhere**: `semisearch "your search" .`

### Too many results?
SemiSearch automatically gives you tips:
```bash
semisearch "function"
# Shows: "Many results found. Use more specific terms or search in specific folders"
```

Try:
- **Be more specific**: `semisearch "function validateUser"`
- **Search in specific folders**: `semisearch "TODO" src/`
- **Use exact phrases**: `semisearch "exact phrase" --exact`

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

The help system will guide you step by step and is perfect for learning.

## Simple Flags

### Handle typos and similar words
```bash
semisearch "databse" --fuzzy
```
This will find "database" even with the typo!

### Find exact matches only
```bash
semisearch "exact phrase" --exact
```

### Show more context around matches
```bash
semisearch "function" --context 2
```

### Get output as JSON (for scripts)
```bash
semisearch "config" --format json
```

## Getting Help

### Check if everything is working
```bash
semisearch status
```
This shows:
- What type of project you're in (Rust, JavaScript, etc.)
- How many files are indexed
- What search capabilities are available
- Helpful tips for your project type

### Interactive help and tutorials
```bash
semisearch help-me
```
Perfect for beginners - guides you through examples step by step.

### Detailed diagnostics
```bash
semisearch doctor
```
Shows detailed information about your system and search capabilities.

### Quick command reference
```bash
semisearch --help
```

### Advanced options (when you're ready)
```bash
semisearch --advanced --help
```

## Progressive Learning

SemiSearch learns with you and gives you tips based on your experience:

### New Users (1-3 searches)
- **Encouraging feedback**: "💡 Great start! Keep exploring"
- **Basic guidance**: Simple tips to help you get started

### Intermediate Users (4-10 searches)  
- **Feature discovery**: "💡 Try --fuzzy for spelling variations"
- **Smart suggestions**: Based on what you're actually searching for

### Experienced Users (11+ searches)
- **Advanced features**: "💡 Try --advanced for more powerful options"
- **Power user tips**: Learn about regex, filtering, and advanced modes

The tool automatically adjusts its suggestions based on how much you've used it.

## Common Use Cases

### For Developers
- `semisearch "TODO"` - Find all TODO comments
- `semisearch "async function"` - Find async functions
- `semisearch "import React"` - Find React imports
- `semisearch "try catch"` - Find error handling
- `semisearch "fn main"` - Find main functions (Rust)
- `semisearch "#[test]"` - Find test functions (Rust)

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
- **Code**: `semisearch "function" src/`
- **Documentation**: `semisearch "tutorial" docs/`
- **Tests**: `semisearch "test" tests/`

### Handle Typos
Always add `--fuzzy` when you're not sure about spelling:
```bash
semisearch "databse connection" --fuzzy
```

## What Makes SemiSearch Special?

### Smart Search
SemiSearch automatically chooses the best search method:
- **Simple words** → Fast keyword search
- **Complex phrases** → Intelligent text analysis
- **Code patterns** → Code-aware search
- **Typos detected** → Automatic fuzzy matching

### Helpful Tips
SemiSearch gives you contextual suggestions:
- When you have no results: Suggests different words or fuzzy search
- When you have too many results: Suggests being more specific
- Based on your experience level: Tips get more advanced as you learn

### Project Awareness
SemiSearch automatically detects what kind of project you're in:
- **Rust projects**: Focuses on .rs files, suggests Rust-specific patterns
- **JavaScript projects**: Focuses on .js/.ts files
- **Documentation projects**: Focuses on .md files
- **Mixed projects**: Adapts to what you're searching

### No Setup Required
Just install and start searching. SemiSearch works immediately and gets better as you use it.

## Making Searches Faster

### Index Large Projects
```bash
semisearch index .
```
This makes all future searches much faster. Do this once for each project.

### Search Specific Folders
```bash
semisearch "function" src/    # Only search src/
semisearch "TODO" tests/      # Only search tests/
```

## Understanding Your Results

SemiSearch organizes results to be helpful:
- **Groups by file**: Shows multiple matches in the same file together
- **Shows context**: You can see the line where each match was found
- **Counts results**: "Found 8 matches" so you know what you're looking at
- **Provides tips**: Suggests what to do if you have too many or too few results

## Next Steps

Once you're comfortable with basic searching:

1. **Try the advanced mode**: `semisearch --advanced --help`
2. **Index your projects**: `semisearch index .` for faster searches
3. **Explore project detection**: `semisearch status` to see how SemiSearch understands your project
4. **Learn from tips**: Pay attention to the suggestions SemiSearch gives you

## Advanced Features (When You're Ready)

### Include/Exclude File Patterns
```bash
semisearch --advanced "TODO" --include "*.rs"     # Only Rust files
semisearch --advanced "test" --exclude "*test*"   # Exclude test files
```

### Specific Search Modes
```bash
semisearch --advanced "pattern.*regex" --mode regex  # Use regex patterns
semisearch --advanced "exact text" --mode keyword    # Exact matching only
```

### Fine-tune Results
```bash
semisearch --advanced "query" --semantic-threshold 0.8  # Higher relevance
```

## Need More Help?

SemiSearch is designed to guide you:

- **Stuck on a search?** The tool will suggest what to try next
- **Want to learn more?** `semisearch help-me` for interactive guidance
- **Need technical details?** `semisearch doctor` for system information
- **Ready for advanced features?** `semisearch --advanced --help`

Remember: SemiSearch grows with you. Start simple, and the tool will teach you more advanced features as you're ready for them!

## Troubleshooting

### "No matches found"
This is normal! SemiSearch will suggest what to try:
- Different words
- Fuzzy search for typos
- Broader search locations

### "Found 400+ matches"
Also normal! SemiSearch will suggest:
- More specific terms
- Searching in specific folders
- Using exact phrases

### Search feels slow
Try:
- `semisearch index .` to speed up future searches
- Search in specific folders instead of everything
- `semisearch doctor` to check system status

### Not finding what you expect
- Try `--fuzzy` for typo tolerance
- Use simpler, more common words
- Check that you're searching in the right directory

Remember: SemiSearch is designed to help you succeed. When something doesn't work, it will tell you exactly what to try next! 