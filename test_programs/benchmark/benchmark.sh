#!/bin/bash

# Default settings
VERBOSE=1
ITERATIONS=100

# Function to display usage information
display_usage() {
    echo "Usage: $0 [options] <program1> <program2>"
    echo "Options:"
    echo "  -v LEVEL   Set verbose level (0=minimal, 1=normal, 2=detailed) [default: 1]"
    echo "  -i NUMBER  Set number of iterations for each benchmark [default: 100]"
    echo "  -h         Display this help message"
    echo ""
    echo "Example: $0 -v 2 -i 50 program1.py program2.out"
    echo "You must specify exactly two programs to benchmark."
}

# Function to print based on verbose level
print_verbose() {
    local level=$1
    local message=$2

    if [ $VERBOSE -ge $level ]; then
        echo "$message"
    fi
}

# Function to measure execution time
measure_time() {
    local program=$1
    local iterations=$2
    print_verbose 1 "Measuring execution time for $program ($iterations iterations)..."

    # Initialize total execution time
    total_time=0

    for ((i=1; i<=$iterations; i++)); do
        # Start time measurement
        start_time=$(date +%s.%N)

        # Execute the program
        if [[ $program == *.py ]]; then
            python3 $program > /dev/null 2>&1
        else
            ./$program > /dev/null 2>&1
        fi

        # End time measurement
        end_time=$(date +%s.%N)

        # Calculate execution time for this iteration
        execution_time=$(echo "$end_time - $start_time" | bc)
        total_time=$(echo "$total_time + $execution_time" | bc)

        print_verbose 2 "Iteration $i: $execution_time seconds"
    done

    # Calculate and print average execution time
    avg_time=$(echo "scale=6; $total_time / $iterations" | bc)
    print_verbose 0 "$program average execution time over $iterations iterations: $avg_time seconds"
    print_verbose 1 "Total execution time: $total_time seconds"
}

# Parse command line arguments
while getopts "v:i:h" opt; do
    case ${opt} in
        v)
            VERBOSE=${OPTARG}
            ;;
        i)
            ITERATIONS=${OPTARG}
            ;;
        h)
            display_usage
            exit 0
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            display_usage
            exit 1
            ;;
    esac
done

# Shift away the processed options
shift $((OPTIND-1))

# Main script
print_verbose 1 "Starting benchmark..."

# Check if exactly two programs are provided
if [ $# -ne 2 ]; then
    echo "Error: You must specify exactly two programs to benchmark." >&2
    display_usage
    exit 1
fi

# Store the two programs
PROGRAM1="$1"
PROGRAM2="$2"

# Run measurements with specified iterations for each program
print_verbose 1 "Running benchmark for first program..."
measure_time "$PROGRAM1" $ITERATIONS

print_verbose 1 "Running benchmark for second program..."
measure_time "$PROGRAM2" $ITERATIONS

print_verbose 1 "Benchmark complete."
