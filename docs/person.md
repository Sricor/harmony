# Person

------------------
## Create
#### `POST` /person/create
###### `Content-Type: application/json`

##### Request Body

| Name     | Type   | Mandatory | Example | Description |
|----------|--------|-----------|---------|-------------|
| name     | String | YES       |         |             |
| password | String | YES       |         |             |

##### Response Body

| Name  | Type   | Mandatory | Example | Description |
|-------|--------|-----------|---------|-------------|
| claim | String | YES       |         |             |



------------------
## Verify
#### `POST` /person/verify
###### `Content-Type: application/json`

##### Request Body

| Name     | Type   | Mandatory | Example | Description |
|----------|--------|-----------|---------|-------------|
| name     | String | YES       |         |             |
| password | String | YES       |         |             |

##### Response Body

| Name  | Type   | Mandatory | Example | Description |
|-------|--------|-----------|---------|-------------|
| claim | String | YES       |         |             |


------------------
## Reissue Claim
#### `POST` /person/verify
###### `Permissions: General`
###### `Content-Type: application/json`


##### Response Body

| Name  | Type   | Mandatory | Example | Description |
|-------|--------|-----------|---------|-------------|
| claim | String | YES       |         |             |
