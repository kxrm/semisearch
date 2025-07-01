#!/bin/bash

# Test script for query analyzer
ANALYZER="./target/debug/analyze"

# Create output files
SEMANTIC_RESULTS="semantic_results.txt"
KEYWORD_RESULTS="keyword_results.txt"

# Clear previous results
> $SEMANTIC_RESULTS
> $KEYWORD_RESULTS

echo "Running semantic query tests..."

# 50 Semantic queries - These should score > 0.5
semantic_queries=(
    # Conceptual relationships
    "relationship between authentication and authorization"
    "how does memory management affect performance"
    "difference between stack and heap allocation"
    "impact of caching on system performance"
    "connection between design patterns and architecture"
    
    # Abstract concepts
    "understanding asynchronous programming concepts"
    "principles of object oriented design"
    "fundamentals of distributed systems"
    "theory behind machine learning algorithms"
    "philosophy of functional programming"
    
    # Multi-concept queries
    "database optimization strategies for large datasets"
    "security implications of cloud computing"
    "scalability challenges in microservices architecture"
    "performance considerations for real-time systems"
    "complexity analysis of sorting algorithms"
    
    # Questions needing understanding
    "what causes memory leaks in applications"
    "how to implement error handling patterns"
    "when to use inheritance versus composition"
    "why immutability matters in concurrent programming"
    "where bottlenecks occur in web applications"
    
    # Technical investigations
    "analyzing network latency issues"
    "debugging race conditions in multithreaded code"
    "evaluating trade-offs in system design"
    "investigating performance degradation causes"
    "exploring alternatives to relational databases"
    
    # Process and methodology
    "best practices for code review process"
    "strategies for refactoring legacy systems"
    "approaches to automated testing"
    "methods for continuous integration"
    "techniques for performance profiling"
    
    # Comparative analysis
    "comparing REST and GraphQL architectures"
    "evaluating different caching strategies"
    "contrasting SQL and NoSQL databases"
    "analyzing various authentication methods"
    "assessing different deployment strategies"
    
    # Problem-solving queries
    "solving concurrency issues in distributed systems"
    "addressing scalability problems in applications"
    "handling edge cases in algorithms"
    "managing state in reactive applications"
    "dealing with eventual consistency"
    
    # System interactions
    "interaction between frontend and backend"
    "communication patterns in microservices"
    "data flow in event-driven architecture"
    "message passing between processes"
    "coordination in distributed transactions"
    
    # Advanced concepts
    "implications of CAP theorem"
    "understanding Byzantine fault tolerance"
    "concepts behind blockchain technology"
    "principles of quantum computing"
    "foundations of cryptographic protocols"
)

# Run semantic queries
for query in "${semantic_queries[@]}"; do
    echo "Testing: $query"
    result=$($ANALYZER "$query" 2>/dev/null | grep "Semantic Score:" | awk '{print $NF}')
    echo "$result" >> $SEMANTIC_RESULTS
done

echo -e "\nRunning keyword query tests..."

# 50 Keyword queries - These should score < 0.5
keyword_queries=(
    # Single terms
    "TODO"
    "README"
    "config"
    "index"
    "login"
    "users"
    "data"
    "test"
    "main"
    "utils"
    
    # File references
    "package.json"
    "index.html"
    "style.css"
    "app.js"
    "main.py"
    "config.yaml"
    "Dockerfile"
    ".gitignore"
    "README.md"
    "LICENSE"
    
    # Identifiers
    "user_id"
    "session_token"
    "api_key"
    "UUID"
    "hash_value"
    "timestamp"
    "version_number"
    "build_id"
    "request_id"
    "transaction_id"
    
    # Simple lookups
    "status"
    "error"
    "success"
    "failed"
    "pending"
    "active"
    "disabled"
    "true"
    "false"
    "null"
    
    # Commands/actions
    "start"
    "stop"
    "restart"
    "enable"
    "disable"
    "create"
    "delete"
    "update"
    "read"
    "write"
)

# Run keyword queries
for query in "${keyword_queries[@]}"; do
    echo "Testing: $query"
    result=$($ANALYZER "$query" 2>/dev/null | grep "Semantic Score:" | awk '{print $NF}')
    echo "$result" >> $KEYWORD_RESULTS
done

echo -e "\n=== ANALYSIS RESULTS ==="

# Calculate statistics
echo -e "\nSemantic Queries Statistics:"
awk '{sum+=$1; sumsq+=$1*$1} END {
    mean=sum/NR; 
    var=sumsq/NR - mean*mean; 
    print "Count: " NR;
    print "Mean: " mean; 
    print "Std Dev: " sqrt(var);
    print "Min: " min;
    print "Max: " max;
}' $SEMANTIC_RESULTS

echo -e "\nKeyword Queries Statistics:"
awk '{sum+=$1; sumsq+=$1*$1} END {
    mean=sum/NR; 
    var=sumsq/NR - mean*mean; 
    print "Count: " NR;
    print "Mean: " mean; 
    print "Std Dev: " sqrt(var);
}' $KEYWORD_RESULTS

# Count how many were correctly classified
echo -e "\nClassification Accuracy:"
echo -n "Semantic queries correctly identified (>0.5): "
awk '$1 > 0.5 {count++} END {print count "/" NR " (" int(count*100/NR) "%)"}' $SEMANTIC_RESULTS

echo -n "Keyword queries correctly identified (≤0.5): "
awk '$1 <= 0.5 {count++} END {print count "/" NR " (" int(count*100/NR) "%)"}' $KEYWORD_RESULTS

# Overall accuracy
total_correct=$(( $(awk '$1 > 0.5 {count++} END {print count}' $SEMANTIC_RESULTS) + $(awk '$1 <= 0.5 {count++} END {print count}' $KEYWORD_RESULTS) ))
total_queries=100
echo -e "\nOverall Accuracy: $total_correct/$total_queries ($(( total_correct * 100 / total_queries ))%)"

# T-test approximation (assuming normal distribution)
echo -e "\nStatistical Significance Test:"
semantic_mean=$(awk '{sum+=$1} END {print sum/NR}' $SEMANTIC_RESULTS)
keyword_mean=$(awk '{sum+=$1} END {print sum/NR}' $KEYWORD_RESULTS)
echo "Semantic mean: $semantic_mean"
echo "Keyword mean: $keyword_mean"
echo "Difference: $(echo "$semantic_mean - $keyword_mean" | bc -l)"

# Distribution analysis
echo -e "\nScore Distribution:"
echo "Semantic queries by score range:"
echo -n "  0.0-0.3: "; awk '$1 >= 0.0 && $1 < 0.3 {count++} END {print count}' $SEMANTIC_RESULTS
echo -n "  0.3-0.5: "; awk '$1 >= 0.3 && $1 < 0.5 {count++} END {print count}' $SEMANTIC_RESULTS
echo -n "  0.5-0.7: "; awk '$1 >= 0.5 && $1 < 0.7 {count++} END {print count}' $SEMANTIC_RESULTS
echo -n "  0.7-1.0: "; awk '$1 >= 0.7 && $1 <= 1.0 {count++} END {print count}' $SEMANTIC_RESULTS

echo "Keyword queries by score range:"
echo -n "  0.0-0.3: "; awk '$1 >= 0.0 && $1 < 0.3 {count++} END {print count}' $KEYWORD_RESULTS
echo -n "  0.3-0.5: "; awk '$1 >= 0.3 && $1 < 0.5 {count++} END {print count}' $KEYWORD_RESULTS
echo -n "  0.5-0.7: "; awk '$1 >= 0.5 && $1 < 0.7 {count++} END {print count}' $KEYWORD_RESULTS
echo -n "  0.7-1.0: "; awk '$1 >= 0.7 && $1 <= 1.0 {count++} END {print count}' $KEYWORD_RESULTS 