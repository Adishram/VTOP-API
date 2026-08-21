#!/bin/bash

# VTOP API Smoke Test
# Usage: ./smoke_test.sh <username> <password> <semester_id>

if [ "$#" -ne 3 ]; then
    echo "Usage: ./smoke_test.sh <username> <password> <semester_id>"
    echo "Example: ./smoke_test.sh 21BCE1234 mypassword CH2025261"
    exit 1
fi

USERNAME=$1
PASSWORD=$2
SEMESTER=$3

API_URL="http://localhost:3000/api"

echo "==========================================="
echo "1. Initiating Login (Fetching Captcha)"
echo "==========================================="

LOGIN_RES=$(curl -s -X POST $API_URL/login -H "Content-Type: application/json" -d "{\"username\": \"$USERNAME\"}")

STATUS=$(echo $LOGIN_RES | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
if [ "$STATUS" != "ok" ]; then
    echo "Failed to initiate login: $LOGIN_RES"
    exit 1
fi

SESSION_ID=$(echo $LOGIN_RES | grep -o '"login_session_id":"[^"]*"' | cut -d'"' -f4)
CAPTCHA_BASE64=$(echo $LOGIN_RES | grep -o '"captcha_image":"[^"]*"' | cut -d'"' -f4 | sed 's/data:image\/png;base64,//')

echo "Login Session ID: $SESSION_ID"
echo "Decoding captcha to captcha.png..."
echo $CAPTCHA_BASE64 | base64 --decode > captcha.png

# Open captcha (Mac only)
open captcha.png

echo ""
read -p "Enter the captcha shown in captcha.png: " CAPTCHA_TEXT

echo "==========================================="
echo "2. Verifying Login"
echo "==========================================="

VERIFY_RES=$(curl -s -X POST $API_URL/login/verify -H "Content-Type: application/json" -d "{
    \"login_session_id\": \"$SESSION_ID\",
    \"username\": \"$USERNAME\",
    \"password\": \"$PASSWORD\",
    \"captcha\": \"$CAPTCHA_TEXT\"
}")

STATUS=$(echo $VERIFY_RES | grep -o '"status":"[^"]*"' | cut -d'"' -f4)
if [ "$STATUS" != "ok" ]; then
    echo "Login failed: $VERIFY_RES"
    exit 1
fi

TOKEN=$(echo $VERIFY_RES | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
echo "JWT Token: $TOKEN"

echo "==========================================="
echo "3. Fetching Live Attendance"
echo "==========================================="

curl -s -X GET "$API_URL/attendance?semester=$SEMESTER" -H "Authorization: Bearer $TOKEN" | grep -o '"status":"ok"' > /dev/null

if [ $? -eq 0 ]; then
    echo "Success! Live attendance fetched."
    curl -s -X GET "$API_URL/attendance?semester=$SEMESTER" -H "Authorization: Bearer $TOKEN"
else
    echo "Failed to fetch attendance."
    curl -s -X GET "$API_URL/attendance?semester=$SEMESTER" -H "Authorization: Bearer $TOKEN"
fi

rm captcha.png
echo ""
