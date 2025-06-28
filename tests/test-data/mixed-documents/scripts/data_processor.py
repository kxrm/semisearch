#!/usr/bin/env python3
"""
Data Processor Script

This script processes CSV data files, performs basic analysis,
and outputs summary statistics.

Usage:
    python data_processor.py [input_file] [output_file]
"""

import sys
import os
import csv
import statistics
from datetime import datetime
from typing import List, Dict, Any, Tuple

def read_csv_file(file_path: str) -> List[Dict[str, Any]]:
    """
    Read data from a CSV file into a list of dictionaries.
    
    Args:
        file_path: Path to the CSV file
        
    Returns:
        List of dictionaries representing the CSV data
    """
    if not os.path.exists(file_path):
        print(f"Error: File '{file_path}' not found.")
        sys.exit(1)
        
    try:
        with open(file_path, 'r', newline='') as csvfile:
            reader = csv.DictReader(csvfile)
            return list(reader)
    except Exception as e:
        print(f"Error reading CSV file: {e}")
        sys.exit(1)

def analyze_data(data: List[Dict[str, Any]]) -> Dict[str, Any]:
    """
    Perform basic analysis on the data.
    
    Args:
        data: List of dictionaries containing the data
        
    Returns:
        Dictionary containing analysis results
    """
    # Initialize results
    results = {
        'total_records': len(data),
        'total_sales': 0,
        'by_region': {},
        'by_product': {},
        'by_month': {}
    }
    
    # Process each record
    for record in data:
        # Convert string values to appropriate types
        try:
            units = int(record.get('Units', 0))
            price = float(record.get('Price', 0))
            total = float(record.get('Total', 0))
            date_str = record.get('Date', '')
            date = datetime.strptime(date_str, '%Y-%m-%d') if date_str else None
            region = record.get('Region', 'Unknown')
            product = record.get('Product', 'Unknown')
        except (ValueError, TypeError) as e:
            print(f"Warning: Data conversion error in record {record}: {e}")
            continue
            
        # Update total sales
        results['total_sales'] += total
        
        # Update region statistics
        if region not in results['by_region']:
            results['by_region'][region] = {'sales': 0, 'units': 0}
        results['by_region'][region]['sales'] += total
        results['by_region'][region]['units'] += units
        
        # Update product statistics
        if product not in results['by_product']:
            results['by_product'][product] = {'sales': 0, 'units': 0}
        results['by_product'][product]['sales'] += total
        results['by_product'][product]['units'] += units
        
        # Update month statistics
        if date:
            month = date.strftime('%Y-%m')
            if month not in results['by_month']:
                results['by_month'][month] = {'sales': 0, 'units': 0}
            results['by_month'][month]['sales'] += total
            results['by_month'][month]['units'] += units
    
    # Calculate averages and percentages
    for region, stats in results['by_region'].items():
        stats['percent'] = (stats['sales'] / results['total_sales']) * 100 if results['total_sales'] > 0 else 0
        
    for product, stats in results['by_product'].items():
        stats['percent'] = (stats['sales'] / results['total_sales']) * 100 if results['total_sales'] > 0 else 0
        
    for month, stats in results['by_month'].items():
        stats['percent'] = (stats['sales'] / results['total_sales']) * 100 if results['total_sales'] > 0 else 0
    
    return results

def write_results(results: Dict[str, Any], output_file: str) -> None:
    """
    Write analysis results to an output file.
    
    Args:
        results: Dictionary containing analysis results
        output_file: Path to the output file
    """
    try:
        with open(output_file, 'w') as f:
            f.write("DATA ANALYSIS RESULTS\n")
            f.write(f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            
            f.write(f"Total Records: {results['total_records']}\n")
            f.write(f"Total Sales: ${results['total_sales']:.2f}\n\n")
            
            f.write("SALES BY REGION:\n")
            for region, stats in sorted(results['by_region'].items(), key=lambda x: x[1]['sales'], reverse=True):
                f.write(f"{region}: ${stats['sales']:.2f} ({stats['percent']:.1f}%) - {stats['units']} units\n")
            
            f.write("\nSALES BY PRODUCT:\n")
            for product, stats in sorted(results['by_product'].items(), key=lambda x: x[1]['sales'], reverse=True):
                f.write(f"{product}: ${stats['sales']:.2f} ({stats['percent']:.1f}%) - {stats['units']} units\n")
            
            f.write("\nSALES BY MONTH:\n")
            for month, stats in sorted(results['by_month'].items()):
                f.write(f"{month}: ${stats['sales']:.2f} ({stats['percent']:.1f}%) - {stats['units']} units\n")
                
        print(f"Results written to {output_file}")
    except Exception as e:
        print(f"Error writing results: {e}")
        sys.exit(1)

def main() -> None:
    """Main function to process the data."""
    # Parse command line arguments
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} [input_file] [output_file]")
        sys.exit(1)
        
    input_file = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else "analysis_results.txt"
    
    # Process the data
    print(f"Reading data from {input_file}...")
    data = read_csv_file(input_file)
    
    print("Analyzing data...")
    results = analyze_data(data)
    
    print(f"Writing results to {output_file}...")
    write_results(results, output_file)
    
    print("Data processing completed successfully!")

if __name__ == "__main__":
    main() 