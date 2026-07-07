#!/usr/bin/env python3
"""
Example usage of the KappaApk HTTP client for authentication and API calls.
"""

import kappa_apk
import json

def main():
    # Create HTTP client with base URL
    base_url = "http://172.16.71.20:8060"
    client = kappa_apk.HttpClient(base_url)
    
    print("=== KappaApk HTTP Client Example ===\n")
    
    # Example 1: Authenticate user
    print("1. Authenticating user...")
    try:
        login_result = client.authenticate("admin", "admin@123")
        print("✓ Authentication successful!")
        print(f"   User: {login_result['user_name']}")
        print(f"   Email: {login_result['email']}")
        print(f"   Token: {login_result['token'][:50]}...")
        print(f"   Organization: {login_result['org_details']['org_name']}")
        print(f"   User Type: {login_result['user_type_details']['user_type']}")
        
        # Store token for subsequent requests
        auth_token = login_result['token']
        
    except Exception as e:
        print(f"✗ Authentication failed: {e}")
        return
    
    print("\n" + "="*50 + "\n")
    
    # Example 2: Make authenticated request
    print("2. Making authenticated request...")
    try:
        # Example: Get current user profile
        response = client.make_request(
            method="GET",
            endpoint="/user-micro-services/v2/users/me",
            data=None,
            token=auth_token
        )
        print("✓ Authenticated request successful!")
        print(f"   Response: {json.dumps(response, indent=2)}")
        
    except Exception as e:
        print(f"✗ Authenticated request failed: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # Example 3: Make POST request with data
    print("3. Making POST request with data...")
    try:
        # Example: Filter datasets
        post_data = {
            "page": 1,
            "size": 10,
            "orderBy": "modifiedOn",
            "orderKeyword": "DESC"
        }

        response = client.make_request(
            method="GET",
            endpoint="/data-micro-services/v2/datasets/filter",
            data=json.dumps(post_data),
            token=auth_token
        )
        print("✓ POST request successful!")
        print(f"   Response: {json.dumps(response, indent=2)}")
        
    except Exception as e:
        print(f"✗ POST request failed: {e}")
    
    print("\n" + "="*50 + "\n")
    
    # Example 4: Error handling
    print("4. Testing error handling...")
    try:
        # Try to authenticate with wrong credentials
        client.authenticate("wrong_user", "wrong_password")
        print("✗ Should have failed with wrong credentials")
        
    except Exception as e:
        print("✓ Correctly handled authentication error")
        print(f"   Error: {e}")

def test_basic_function():
    """Test the basic sum_as_string function"""
    print("Testing basic function...")
    result = kappa_apk.sum_as_string(5, 3)
    print(f"sum_as_string(5, 3) = {result}")
    assert result == "8"
    print("✓ Basic function test passed!")

if __name__ == "__main__":
    # Test basic functionality first
    test_basic_function()
    print("\n" + "="*50 + "\n")
    
    # Run main example
    main() 