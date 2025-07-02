# SemiSearch for Beginners

## What is SemiSearch?

SemiSearch helps you find text in your files, even when you don't remember the exact words. It automatically uses the best search strategy for your query:

- **Simple terms** get fast keyword search
- **Conceptual queries** automatically use semantic search to find related meanings
- **Poor results** trigger automatic semantic fallback

The tool is designed to be simple for beginners but grows with you as you learn.

## Installation

You need to build SemiSearch from source:

```bash
# Clone the repository
git clone https://github.com/kxrm/semisearch.git
cd semisearch

# Build the tool
cargo build --release --features neural-embeddings

# The binary will be at target/release/semisearch
./target/release/semisearch --help
```

## Basic Usage

Just type what you're looking for:
```bash
./target/release/semisearch "what you want to find"
```

That's it! No configuration required.

## Examples

### Find TODO comments
```bash
./target/release/semisearch "TODO"
```

### Find database-related code
```bash
./target/release/semisearch "database"
```

### Find error handling
```bash
./target/release/semisearch "error"
```

### Find function definitions
```bash
./target/release/semisearch "function login"
```

### Find concepts automatically
```bash
./target/release/semisearch "authentication"     # Finds login, auth, user validation, etc.
./target/release/semisearch "error handling"     # Finds try/catch, exceptions, error recovery
```

### Search in specific places
```bash
./target/release/semisearch "TODO" src/          # Search only in src/ directory
./target/release/semisearch "config" config.json # Search in specific file
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
- **Check spelling**: `./target/release/semisearch "databse" --fuzzy` (finds "database")
- **Use simpler words**: `./target/release/semisearch "login"` instead of `./target/release/semisearch "authentication"`
- **Search everywhere**: `./target/release/semisearch "your search" .`

### Too many results?
SemiSearch automatically gives you tips:
```bash
./target/release/semisearch "function"
# Shows: "Many results found. Use more specific terms or search in specific folders"
```

Try:
- **Be more specific**: `./target/release/semisearch "function validateUser"`
- **Search in specific folders**: `./target/release/semisearch "TODO" src/`
- **Use exact phrases**: `./target/release/semisearch "exact phrase" --exact`

### Still stuck?
Run: `./target/release/semisearch help-me` for interactive help.

## Interactive Help

SemiSearch includes an interactive help system that guides you through common scenarios:

```bash
./target/release/semisearch help-me
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
./target/release/semisearch "databse" --fuzzy
```
This will find "database" even with the typo!

### Find exact matches only
```bash
./target/release/semisearch "exact phrase" --exact
```

### Show more context around matches
```bash
./target/release/semisearch --advanced "function" --context 2
```

### Find related concepts automatically
```bash
./target/release/semisearch "authentication"
# Automatically finds: login, auth, user validation, credentials, etc.
```

### Get output as JSON (for scripts)
```bash
./target/release/semisearch --advanced "config" --format json
```

## Getting Help

### Check if everything is working
```bash
./target/release/semisearch status
```
This shows:
- What type of project you're in (Rust, JavaScript, etc.)
- How many files are indexed
- What search capabilities are available
- Helpful tips for your project type

### Interactive help and tutorials
```bash
./target/release/semisearch help-me
```
Perfect for beginners - guides you through examples step by step.

### Detailed diagnostics
```bash
./target/release/semisearch doctor
```
Shows detailed information about your system and search capabilities.

### Quick command reference
```bash
./target/release/semisearch --help
```

### Advanced options (when you're ready)
```bash
./target/release/semisearch --advanced --help
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
- `./target/release/semisearch "TODO"` - Find all TODO comments
- `./target/release/semisearch "async function"` - Find async functions
- `./target/release/semisearch "import React"` - Find React imports
- `./target/release/semisearch "try catch"` - Find error handling
- `./target/release/semisearch "fn main"` - Find main functions (Rust)
- `./target/release/semisearch "#[test]"` - Find test functions (Rust)

### For Writers
- `./target/release/semisearch "needs revision"` - Find draft sections
- `./target/release/semisearch "citation needed"` - Find unsourced claims
- `./target/release/semisearch "methodology"` - Find research methods

### For Configuration
- `./target/release/semisearch "password"` - Find password-related items
- `./target/release/semisearch "config"` - Find configuration files
- `./target/release/semisearch "port 8080"` - Find port configurations

## Tips for Better Results

### Start Simple, Then Get Specific
1. Start with: `./target/release/semisearch "login"`
2. If too many results: `./target/release/semisearch "login function"`
3. If still too many: `./target/release/semisearch "function validateLogin"`

### Use Different Words
If "error" doesn't work, try:
- `./target/release/semisearch "exception"`
- `./target/release/semisearch "fail"`
- `./target/release/semisearch "catch"`

### Search in the Right Place
- **Code**: `./target/release/semisearch "function" src/`
- **Documentation**: `./target/release/semisearch "tutorial" docs/`
- **Tests**: `./target/release/semisearch "test" tests/`

### Handle Typos
Always add `--fuzzy` when you're not sure about spelling:
```bash
./target/release/semisearch "databse connection" --fuzzy
```

## What Makes SemiSearch Special?

### Automatic Smart Search
SemiSearch automatically chooses the best search method:
- **Simple terms** → Fast keyword search
- **Conceptual queries** → Semantic search finds related meanings
- **Poor keyword results** → Automatic semantic fallback
- **Code patterns** → Code-aware search
- **Typos detected** → Automatic fuzzy matching

### Examples of Automatic Behavior
```bash
./target/release/semisearch "TODO"           # Fast keyword search
./target/release/semisearch "authentication" # Automatic semantic search
./target/release/semisearch "user login"     # Keyword first, semantic if needed
```

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
./target/release/semisearch index .
```
This makes all future searches much faster. Do this once for each project.

### Search Specific Folders
```bash
./target/release/semisearch "function" src/    # Only search src/
./target/release/semisearch "TODO" tests/      # Only search tests/
```

## Understanding Your Results

SemiSearch organizes results to be helpful:
- **Groups by file**: Shows multiple matches in the same file together
- **Shows context**: You can see the line where each match was found
- **Counts results**: "Found 8 matches" so you know what you're looking at
- **Provides tips**: Suggests what to do if you have too many or too few results

## Next Steps

Once you're comfortable with basic searching:

1. **Try the advanced mode**: `./target/release/semisearch --advanced --help`
2. **Index your projects**: `./target/release/semisearch index .` for faster searches
3. **Explore project detection**: `./target/release/semisearch status` to see how SemiSearch understands your project
4. **Learn from tips**: Pay attention to the suggestions SemiSearch gives you

## Advanced Features (When You're Ready)

### Include/Exclude File Patterns
```bash
./target/release/semisearch --advanced "TODO" --include "*.rs"     # Only Rust files
./target/release/semisearch --advanced "test" --exclude "*test*"   # Exclude test files
```

### Specific Search Modes
```bash
./target/release/semisearch --advanced "pattern.*regex" --mode regex     # Use regex patterns
./target/release/semisearch --advanced "authentication" --mode semantic  # Semantic/conceptual search
./target/release/semisearch --advanced "exact text" --mode keyword       # Exact matching only
```

### Context and Output Options
```bash
./target/release/semisearch --advanced "function" --context 3            # Show surrounding lines
./target/release/semisearch --advanced "config" --format json            # JSON output
./target/release/semisearch --advanced "TODO" --files-only               # Show only file paths
```

### Fine-tune Results
```bash
./target/release/semisearch --advanced "query" --semantic-threshold 0.8  # Higher relevance
```

## Need More Help?

SemiSearch is designed to guide you:

- **Stuck on a search?** The tool will suggest what to try next
- **Want to learn more?** `./target/release/semisearch help-me` for interactive guidance
- **Need technical details?** `./target/release/semisearch doctor` for system information
- **Ready for advanced features?** `./target/release/semisearch --advanced --help`

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
- `./target/release/semisearch index .` to speed up future searches
- Search in specific folders instead of everything
- `./target/release/semisearch doctor` to check system status

### Not finding what you expect
- Try `--fuzzy` for typo tolerance
- Use simpler, more common words
- Check that you're searching in the right directory

Remember: SemiSearch is designed to help you succeed. When something doesn't work, it will tell you exactly what to try next! 