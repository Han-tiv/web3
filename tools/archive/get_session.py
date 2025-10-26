#!/usr/bin/env python3

import requests
import pyotp
import json
import sys
import re
from urllib.parse import quote

class TwitterSessionCreator:
    def __init__(self):
        self.session = requests.Session()
        self.session.headers.update({
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
            'Accept': 'application/json, text/javascript, */*; q=0.01',
            'Accept-Language': 'en-US,en;q=0.5',
            'Accept-Encoding': 'gzip, deflate, br',
            'DNT': '1',
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1'
        })

    def get_guest_token(self):
        """获取guest token"""
        try:
            bearer_token = "AAAAAAAAAAAAAAAAAAAAANRILgAAAAAAnNwIzUejRCOuH5E6I8xnZz4puTs%3D1Zv7ttfk8LF81IUq16cHjhLTvJu4FA33AGWWjCpTnA"

            headers = {
                'Authorization': f'Bearer {bearer_token}',
                'Content-Type': 'application/x-www-form-urlencoded',
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/guest/activate.json',
                headers=headers
            )

            if response.status_code == 200:
                guest_token = response.json().get('guest_token')
                self.session.headers['x-guest-token'] = guest_token
                self.session.headers['Authorization'] = f'Bearer {bearer_token}'
                return guest_token
            else:
                print(f"Failed to get guest token: {response.status_code}")
                return None

        except Exception as e:
            print(f"Error getting guest token: {e}")
            return None

    def login_step1(self, username, password):
        """第一步登录"""
        try:
            flow_data = {
                "flow_name": "login",
                "input_flow_data": {
                    "flow_context": {
                        "debug_overrides": {},
                        "start_location": {
                            "location": "splash_screen"
                        }
                    }
                }
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/onboarding/task.json?flow_name=login',
                json=flow_data
            )

            if response.status_code != 200:
                print(f"Login step 1 failed: {response.status_code}")
                print(response.text)
                return None

            data = response.json()
            flow_token = data.get('flow_token')

            # 第二步 - 提供用户名
            username_data = {
                "flow_token": flow_token,
                "subtask_inputs": [{
                    "subtask_id": "LoginJsInstrumentationSubtask",
                    "js_instrumentation": {
                        "response": "{}",
                        "link": "next_link"
                    }
                }]
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/onboarding/task.json',
                json=username_data
            )

            if response.status_code != 200:
                print(f"Username step failed: {response.status_code}")
                return None

            data = response.json()
            flow_token = data.get('flow_token')

            # 第三步 - 输入用户名
            enter_username_data = {
                "flow_token": flow_token,
                "subtask_inputs": [{
                    "subtask_id": "LoginEnterUserIdentifierSSO",
                    "settings_list": {
                        "setting_responses": [{
                            "key": "user_identifier",
                            "response_data": {
                                "text_data": {
                                    "result": username
                                }
                            }
                        }],
                        "link": "next_link"
                    }
                }]
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/onboarding/task.json',
                json=enter_username_data
            )

            if response.status_code != 200:
                print(f"Enter username failed: {response.status_code}")
                return None

            data = response.json()
            flow_token = data.get('flow_token')

            # 第四步 - 输入密码
            password_data = {
                "flow_token": flow_token,
                "subtask_inputs": [{
                    "subtask_id": "LoginEnterPassword",
                    "enter_password": {
                        "password": password,
                        "link": "next_link"
                    }
                }]
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/onboarding/task.json',
                json=password_data
            )

            if response.status_code != 200:
                print(f"Password step failed: {response.status_code}")
                print(response.text)
                return None

            data = response.json()

            # 检查是否需要2FA
            subtasks = data.get('subtasks', [])
            for subtask in subtasks:
                if subtask.get('subtask_id') == 'LoginTwoFactorAuthChallenge':
                    return {'flow_token': data.get('flow_token'), 'requires_2fa': True}

            # 如果不需要2FA，提取tokens
            return self.extract_tokens(data)

        except Exception as e:
            print(f"Login error: {e}")
            return None

    def handle_2fa(self, flow_token, totp_secret):
        """处理2FA"""
        try:
            totp = pyotp.TOTP(totp_secret)
            code = totp.now()

            twofa_data = {
                "flow_token": flow_token,
                "subtask_inputs": [{
                    "subtask_id": "LoginTwoFactorAuthChallenge",
                    "enter_text": {
                        "text": code,
                        "link": "next_link"
                    }
                }]
            }

            response = self.session.post(
                'https://api.twitter.com/1.1/onboarding/task.json',
                json=twofa_data
            )

            if response.status_code != 200:
                print(f"2FA failed: {response.status_code}")
                print(response.text)
                return None

            data = response.json()
            return self.extract_tokens(data)

        except Exception as e:
            print(f"2FA error: {e}")
            return None

    def extract_tokens(self, data):
        """从响应中提取OAuth tokens"""
        try:
            # 从cookies中提取tokens
            cookies = self.session.cookies.get_dict()

            oauth_token = None
            oauth_token_secret = None

            # 尝试从不同来源提取
            for cookie_name, cookie_value in cookies.items():
                if 'oauth_token' in cookie_name.lower():
                    oauth_token = cookie_value
                elif 'oauth_secret' in cookie_name.lower():
                    oauth_token_secret = cookie_value

            # 备选方案：从响应数据中查找
            if not oauth_token or not oauth_token_secret:
                response_text = json.dumps(data)

                # 使用正则表达式查找token模式
                token_patterns = [
                    r'"oauth_token":"([^"]+)"',
                    r'"oauth_token_secret":"([^"]+)"',
                    r'"token":"([^"]+)"',
                    r'"secret":"([^"]+)"'
                ]

                for pattern in token_patterns:
                    matches = re.findall(pattern, response_text)
                    if matches and not oauth_token:
                        oauth_token = matches[0]
                    elif matches and not oauth_token_secret:
                        oauth_token_secret = matches[0]

            if oauth_token and oauth_token_secret:
                return {
                    'oauth_token': oauth_token,
                    'oauth_token_secret': oauth_token_secret
                }
            else:
                print("Failed to extract OAuth tokens")
                print("Available cookies:", list(cookies.keys()))
                return None

        except Exception as e:
            print(f"Token extraction error: {e}")
            return None

def main():
    if len(sys.argv) != 5:
        print("Usage: python get_session.py <username> <password> <2fa_secret> <output_file>")
        sys.exit(1)

    username = sys.argv[1]
    password = sys.argv[2]
    totp_secret = sys.argv[3]
    output_file = sys.argv[4]

    print(f"Creating session for user: {username}")

    creator = TwitterSessionCreator()

    # 获取guest token
    print("Getting guest token...")
    guest_token = creator.get_guest_token()
    if not guest_token:
        print("Failed to get guest token")
        sys.exit(1)

    print(f"Guest token obtained: {guest_token[:20]}...")

    # 登录
    print("Logging in...")
    login_result = creator.login_step1(username, password)

    if not login_result:
        print("Login failed")
        sys.exit(1)

    # 处理2FA
    if login_result.get('requires_2fa'):
        print("2FA required, processing...")
        tokens = creator.handle_2fa(login_result['flow_token'], totp_secret)
    else:
        tokens = login_result

    if not tokens or 'oauth_token' not in tokens:
        print("Failed to get OAuth tokens")
        sys.exit(1)

    # 保存到文件
    print(f"Saving session to {output_file}")
    with open(output_file, 'w') as f:
        json.dump(tokens, f)
        f.write('\n')

    print("Session created successfully!")
    print(f"OAuth Token: {tokens['oauth_token'][:20]}...")
    print(f"OAuth Secret: {tokens['oauth_token_secret'][:20]}...")

if __name__ == "__main__":
    main()