# Binace

------------------
## Balance
#### `GET` /binance/balance
###### `Permissions: General`
###### `Content-Type: application/json`



------------------
## Secret
#### `GET` /binance/secret
###### `Permissions: General`
###### `Content-Type: application/json`

##### Response Body

| Name | Type                  | Mandatory | Example | Description |
|------|-----------------------|-----------|---------|-------------|
| -    | Array[Object[Secret]] | YES       |         |             |

------------------
## Add Secret
#### `POST` /binance/secret
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name | Type           | Mandatory | Example | Description |
|------|----------------|-----------|---------|-------------|
| -    | Object[Secret] | YES       |         |             |



------------------
## Spot Inventory
#### `GET` /binance/spot
###### `Permissions: General`
###### `Content-Type: application/json`

##### Response Body

| Name | Type                | Mandatory | Example | Description |
|------|---------------------|-----------|---------|-------------|
| -    | Array[Object[Spot]] | YES       |         |             |


------------------
## Spot Insert
#### `POST` /binance/spot
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name | Type         | Mandatory | Example | Description |
|------|--------------|-----------|---------|-------------|
| -    | Object[Spot] | YES       |         |             |



------------------
## Spot Predict
#### `POST` /binance/spot/predict
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name    | Type            | Mandatory | Example | Description |
|---------|-----------------|-----------|---------|-------------|
| symbol  | String          | YES       |         |             |
| trading | Object[Trading] | YES       |         |             |


##### Response Body

| Name              | Type            | Mandatory | Example | Description |
|-------------------|-----------------|-----------|---------|-------------|
| buying            | Object[Buying]  | YES       |         |             |
| selling           | Object[Selling] | YES       |         |             |
| net_profit        | Decimal         | YES       |         |             |
| net_profit_margin | Decimal         | YES       |         |             |
| leave_quantity    | Decimal         | YES       |         |             |


------------------
## Spot Buy
#### `POST` /binance/spot/buy
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name       | Type    | Mandatory | Example | Description    |
|------------|---------|-----------|---------|----------------|
| symbol     | String  | YES       |         |                |
| investment | Decimal | YES       | "100.0" |                |
| production | Boolean | YES       | false   | Default: false |


##### Response Body

| Name   | Type           | Mandatory | Example | Description |
|--------|----------------|-----------|---------|-------------|
| buying | Object[Buying] | YES       |         |             |



------------------
## Spot Sell
#### `POST` /binance/spot/sell
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name       | Type    | Mandatory | Example | Description    |
|------------|---------|-----------|---------|----------------|
| symbol     | String  | YES       |         |                |
| quantity   | Decimal | YES       | "2.50"  |                |
| production | Boolean | YES       | false   | Default: false |

##### Response Body

| Name    | Type            | Mandatory | Example | Description |
|---------|-----------------|-----------|---------|-------------|
| selling | Object[Selling] | YES       |         |             |



------------------
## Spot Order
#### `GET` /binance/spot/order
###### `Permissions: General`
###### `Content-Type: application/json`

##### Response Body

| Name    | Type                   | Mandatory | Example | Description        |
|---------|------------------------|-----------|---------|--------------------|
| buying  | Array[Object[Buying]]  | YES       |         | spot buying order  |
| selling | Array[Object[Selling]] | YES       |         | spot selling order |



------------------
## Spot Limit
#### `GET` /binance/spot/limit
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name       | Type   | Mandatory | Example | Description                   |
|------------|--------|-----------|---------|-------------------------------|
| identifier | String | YES       |         | binance spot limit identifier |

##### Response Body

| Name  | Type          | Mandatory | Example | Description |
|-------|---------------|-----------|---------|-------------|
| limit | Object[Limit] | YES       |         |             |


------------------
## Spot Limit
#### `POST` /binance/spot/limit
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name | Type          | Mandatory | Example | Description        |
|------|---------------|-----------|---------|--------------------|
| -    | Object[Limit] | YES       |         | binance spot limit |

##### Response Body

| Name       | Type   | Mandatory | Example | Description                   |
|------------|--------|-----------|---------|-------------------------------|
| identifier | String | YES       |         | binance spot limit identifier |


------------------
## Spot Limit
#### `DELETE` /binance/spot/limit
###### `Permissions: General`
###### `Content-Type: application/json`

##### Request Body

| Name       | Type   | Mandatory | Example | Description                   |
|------------|--------|-----------|---------|-------------------------------|
| identifier | String | YES       |         | binance spot limit identifier |


# Object

### Secret

| Name       | Type    | Mandatory | Example  | Description     |
|------------|---------|-----------|----------|-----------------|
| purview    | Integer | YES       | 1        | 1: Read 2: Spot |
| api_key    | String  | YES       | "api"    |                 |
| secret_key | String  | YES       | "secret" |                 |

### Spot

| Name                           | Type    | Mandatory | Example   | Description |
|--------------------------------|---------|-----------|-----------|-------------|
| symbol                         | String  | YES       | "BTCUSDT" |             |
| transaction_quantity_precision | Integer | YES       | 5         |             |
| quantity_precision             | Integer | YES       | 8         |             |
| amount_precision               | Integer | YES       | 8         |             |
| buying_commission              | Decimal | YES       | "0.001"   |             |
| selling_commission             | Decimal | YES       | "0.001"   |             |
| minimum_transaction_amount     | Decimal | YES       | "5.0"     |             |

### Trading

| Name          | Type    | Mandatory | Example | Description |
|---------------|---------|-----------|---------|-------------|
| investment    | Decimal | YES       | "50.0"  |             |
| buying_price  | Decimal | YES       | "100.0" |             |
| selling_price | Decimal | YES       | "200.0" |             |

### Buying

| Name                      | Type    | Mandatory | Example | Description |
|---------------------------|---------|-----------|---------|-------------|
| price                     | Decimal | YES       | "50.0"  |             |
| quantity                  | Decimal | YES       | "2.0"   |             |
| spent                     | Decimal | YES       | "100.0" |             |
| quantity_after_commission | Decimal | YES       | "95.0"  |             |

### Selling

| Name                    | Type    | Mandatory | Example | Description |
|-------------------------|---------|-----------|---------|-------------|
| price                   | Decimal | YES       | "50.0"  |             |
| quantity                | Decimal | YES       | "2.0"   |             |
| income                  | Decimal | YES       | "100.0" |             |
| income_after_commission | Decimal | YES       | "95.0"  |             |



### Limit 

| Name         | Type    | Mandatory | Example   | Description               |
|--------------|---------|-----------|-----------|---------------------------|
| symbol       | String  | YES       | "BTCUSDT" |                           |
| buying_low   | Decimal | YES       | "100.0"   |                           |
| buying_high  | Decimal | YES       | "120.0"   |                           |
| selling_low  | Decimal | YES       | "150.0"   |                           |
| selling_high | Decimal | YES       | "200.0"   |                           |
| investment   | Decimal | YES       | "50"      |                           |
| position     | Decimal | YES       | "5"       | quantity you already have |
| interval     | Integer | YES       | "3"       | every seconds             |
