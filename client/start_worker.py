import requests
import base64
import json
import sys

if len(sys.argv) < 4:
    print('Usage: python %s <url> <img_file> <hashtag> ...' % sys.argv[0])
    quit()

url = sys.argv[1]
img_file_path = sys.argv[2]
hashtags = []
for i in range(3, len(sys.argv)):
    hashtags.append(sys.argv[i])

print('Reading image file %s' % img_file_path)
img = open(img_file_path, 'rb').read()
encoded_img = base64.standard_b64encode(img)
print('Encoded image into base64')

payload = {'origin': encoded_img.decode('utf-8'), 'hashtags': hashtags, 'piece_size': [30, 30]}

res = requests.post(url, data=json.dumps(payload), headers={'Content-Type': 'application/json'})

print(res.status_code)
print(res.text)
