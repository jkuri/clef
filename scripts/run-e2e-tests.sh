#!/bin/bash

# PNRS End-to-End Test Runner
# This script runs comprehensive e2e tests for PNRS with npm, pnpm, and yarn

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check Rust/Cargo
    if ! command_exists cargo; then
        print_error "Cargo is not installed. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check Node.js/npm
    if ! command_exists node; then
        print_error "Node.js is not installed. Please install Node.js: https://nodejs.org/"
        exit 1
    fi
    
    if ! command_exists npm; then
        print_error "npm is not installed. Please install Node.js: https://nodejs.org/"
        exit 1
    fi
    
    print_success "Rust and Node.js are available"
    
    # Check optional package managers
    if command_exists pnpm; then
        print_success "pnpm is available"
    else
        print_warning "pnpm is not installed. Some tests will be skipped. Install with: npm install -g pnpm"
    fi
    
    if command_exists yarn; then
        print_success "yarn is available"
    else
        print_warning "yarn is not installed. Some tests will be skipped. Install with: npm install -g yarn"
    fi
}

# Function to run specific test module
run_test_module() {
    local module=$1
    local description=$2
    
    print_status "Running $description tests..."
    
    if cargo test --test e2e_tests "$module" -- --nocapture; then
        print_success "$description tests passed"
        return 0
    else
        print_error "$description tests failed"
        return 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_status "Running all e2e tests..."
    
    local failed_modules=()
    
    # Core functionality tests
    run_test_module "package_management" "Package Management" || failed_modules+=("Package Management")
    run_test_module "authentication" "Authentication" || failed_modules+=("Authentication")
    run_test_module "publishing" "Publishing" || failed_modules+=("Publishing")
    run_test_module "cache_management" "Cache Management" || failed_modules+=("Cache Management")
    run_test_module "analytics" "Analytics" || failed_modules+=("Analytics")
    run_test_module "security" "Security" || failed_modules+=("Security")
    
    # Advanced feature tests
    run_test_module "scoped_packages" "Scoped Packages" || failed_modules+=("Scoped Packages")
    run_test_module "compatibility" "Cross-Manager Compatibility" || failed_modules+=("Cross-Manager Compatibility")
    run_test_module "performance" "Performance" || failed_modules+=("Performance")
    
    # Summary
    if [ ${#failed_modules[@]} -eq 0 ]; then
        print_success "All e2e test modules passed!"
        return 0
    else
        print_error "The following test modules failed:"
        for module in "${failed_modules[@]}"; do
            echo "  - $module"
        done
        return 1
    fi
}

# Function to run quick tests (subset)
run_quick_tests() {
    print_status "Running quick e2e tests (core functionality only)..."
    
    local failed_modules=()
    
    run_test_module "package_management" "Package Management" || failed_modules+=("Package Management")
    run_test_module "authentication" "Authentication" || failed_modules+=("Authentication")
    run_test_module "cache_management" "Cache Management" || failed_modules+=("Cache Management")
    
    if [ ${#failed_modules[@]} -eq 0 ]; then
        print_success "Quick e2e tests passed!"
        return 0
    else
        print_error "Some quick tests failed"
        return 1
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help              Show this help message"
    echo "  -a, --all               Run all e2e tests (default)"
    echo "  -q, --quick             Run quick tests (core functionality only)"
    echo "  -m, --module MODULE     Run specific test module"
    echo "  -l, --list              List available test modules"
    echo "  -v, --verbose           Enable verbose output"
    echo "  --debug                 Enable debug logging"
    echo ""
    echo "Available modules:"
    echo "  package_management      Package installation and metadata"
    echo "  authentication          User login and authentication"
    echo "  publishing              Package publishing workflows"
    echo "  cache_management        Cache operations and statistics"
    echo "  analytics               Package analytics and database features"
    echo "  security                Security audits and advisories"
    echo "  scoped_packages         Scoped package handling"
    echo "  compatibility           Cross-package-manager compatibility"
    echo "  performance             Performance and load testing"
    echo ""
    echo "Examples:"
    echo "  $0                      # Run all tests"
    echo "  $0 --quick              # Run quick tests"
    echo "  $0 -m package_management # Run package management tests"
    echo "  $0 --debug              # Run with debug logging"
}

# Parse command line arguments
VERBOSE=false
DEBUG=false
MODULE=""
QUICK=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -a|--all)
            # Default behavior, no action needed
            shift
            ;;
        -q|--quick)
            QUICK=true
            shift
            ;;
        -m|--module)
            MODULE="$2"
            shift 2
            ;;
        -l|--list)
            echo "Available test modules:"
            echo "  package_management"
            echo "  authentication"
            echo "  publishing"
            echo "  cache_management"
            echo "  analytics"
            echo "  security"
            echo "  scoped_packages"
            echo "  compatibility"
            echo "  performance"
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --debug)
            DEBUG=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_status "PNRS End-to-End Test Runner"
    print_status "============================"
    
    # Check prerequisites
    check_prerequisites
    
    # Set environment variables
    if [ "$DEBUG" = true ]; then
        export RUST_LOG=debug
        print_status "Debug logging enabled"
    fi
    
    # Change to project root directory
    cd "$(dirname "$0")/.."
    
    # Build the project first
    print_status "Building PNRS..."
    if ! cargo build; then
        print_error "Failed to build PNRS"
        exit 1
    fi
    print_success "Build completed"
    
    # Run tests based on options
    if [ -n "$MODULE" ]; then
        case $MODULE in
            package_management)
                run_test_module "$MODULE" "Package Management"
                ;;
            authentication)
                run_test_module "$MODULE" "Authentication"
                ;;
            publishing)
                run_test_module "$MODULE" "Publishing"
                ;;
            cache_management)
                run_test_module "$MODULE" "Cache Management"
                ;;
            analytics)
                run_test_module "$MODULE" "Analytics"
                ;;
            security)
                run_test_module "$MODULE" "Security"
                ;;
            scoped_packages)
                run_test_module "$MODULE" "Scoped Packages"
                ;;
            compatibility)
                run_test_module "$MODULE" "Cross-Manager Compatibility"
                ;;
            performance)
                run_test_module "$MODULE" "Performance"
                ;;
            *)
                print_error "Unknown module: $MODULE"
                print_status "Use --list to see available modules"
                exit 1
                ;;
        esac
    elif [ "$QUICK" = true ]; then
        run_quick_tests
    else
        run_all_tests
    fi
    
    exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        print_success "E2E tests completed successfully!"
    else
        print_error "E2E tests failed!"
    fi
    
    exit $exit_code
}

# Run main function
main "$@"
