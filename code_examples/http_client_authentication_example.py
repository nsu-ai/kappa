#!/usr/bin/env python3
"""
Example usage of the kf-sdk client for authentication and API calls.

Run::

    python http_client_authentication_example.py \\
        --base-url http://127.0.0.1:8060 --login-id user@example.com --password '***'
"""

from __future__ import annotations

import argparse
import getpass
import json
import sys

import kappa_apk
from kappa_apk import KappaApkClient


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="kf-sdk authentication and API call example",
    )
    parser.add_argument(
        "--base-url",
        default="http://127.0.0.1:8060",
        help="Kappa API gateway base URL (default: %(default)s)",
    )
    parser.add_argument(
        "--login-id",
        required=True,
        help="Username or email for login",
    )
    parser.add_argument(
        "--password",
        default=None,
        help="Password (prompted securely if omitted)",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    password = args.password if args.password is not None else getpass.getpass("Password: ")
    if not password:
        print("✗ Password is required", file=sys.stderr)
        return 1

    client = KappaApkClient(args.base_url, args.login_id, password)

    print("=== kf-sdk HTTP Client Example ===\n")

    print("1. Authenticating user...")
    try:
        login_result = client.connect()
        print("✓ Authentication successful!")
        print(f"   User: {login_result['user_name']}")
        print(f"   Email: {login_result['email']}")
        token = login_result.get("token") or ""
        print(f"   Token: {token[:50]}..." if token else "   Token: (none)")
        org = login_result.get("org_details")
        org_name = org.get("org_name") if org else None
        print(f"   Organization: {org_name or '(none)'}")
        ut = login_result.get("user_type_details") or {}
        print(f"   User Type: {ut.get('user_type', '(unknown)')}")
    except Exception as e:
        print(f"✗ Authentication failed: {e}")
        return 1

    print("\n" + "=" * 50 + "\n")

    print("2. Making authenticated request...")
    try:
        response = client.make_request(
            method="GET",
            endpoint="/user-micro-services/v2/users/me",
            data=None,
            token=None,
        )
        print("✓ Authenticated request successful!")
        print(f"   Response: {json.dumps(response, indent=2)}")
    except Exception as e:
        print(f"✗ Authenticated request failed: {e}")

    print("\n" + "=" * 50 + "\n")

    print("3. Listing datasets (filter)...")
    try:
        response = client.list_datasets(
            page=1, size=10, order_by="modifiedOn", order_keyword="DESC"
        )
        print("✓ Dataset list successful!")
        print(f"   Response: {json.dumps(response, indent=2)}")
    except Exception as e:
        print(f"✗ Dataset list failed: {e}")

    print("\n" + "=" * 50 + "\n")

    print("4. Closing session...")
    try:
        client.close()
        print("✓ Session closed")
    except Exception as e:
        print(f"✗ Close failed: {e}")
        return 1

    return 0


if __name__ == "__main__":
    print(f"kappa_apk version: {kappa_apk.version()}\n")
    print("=" * 50 + "\n")
    sys.exit(main())
