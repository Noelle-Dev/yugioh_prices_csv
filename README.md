# Yugioh Prices CSV
Simple command line tool to price your Yu-Gi-Oh cards. Input and output is in the form of CSV file. 
Prices are retrieved from [YugiohPrices](http://yugiohprices.com).

# Use-Case
1. Tracking value of opened booster boxes/cases
2. Tracking value of your deck

# Command Line Arguments
1. `-f <path/to/file>` - (Optional) path to input file. If not present, it defaults to STDIN.
2. `-o <path/to/file>` - (Optional) path to output file. If not present, it defaults to STDOUT.
3. `-a ArbitrationStrategy` - (Optional) Arbitration strategy to use when input is ambiguous. Values are:
    1. Min/MinValue - Uses the option with the lowest price, this is the default.
    2. Max/MaxValue - Uses the option with the highest price.
4. `--print-total` - (Optional) flag to print the card's total value to STDOUT.

# Input/Output Data Format
CSV is the input/output data format. Headers are mandatory.

Required fields:

1. name - The **name** of the card (Blue-Eyes White Dragon, I:P Masquerena, etc)

Optional fields:

1. tag - The card's **print tag** (LOB-EN001, CHIM-EN049, etc). If missing and multiple versions exist, we choose
   the option that best fits the arbitration strategy.
2. rarity - In the case that a card's tag is ambiguous, rarity is used for arbitration, otherwise first result is used. An example 
   would be I:P Masquerena (CHIM-EN049) that comes in either Ultra Rare, or Starlight Rare
3. count - The amount of the card, defaults to 1. This value is used in calculation of total value
4. price - Price of card, the value is not used for input. This just means that you can use the output as future input (i.e. when 
   price changes the next day)

# Examples
**With complete input (name, tag, rarity)**
```
yugioh_prices_csv -f examples/example.csv
```

Input CSV:
```
tag,name,count,rarity
CHIM-EN049,I:P Masquerena,1,Starlight Rare
CHIM-EN049,I:P Masquerena,2,Ultra Rare
```

Output CSV:
```
tag,name,count,rarity,price
CHIM-EN049,I:P Masquerena,1,Starlight Rare,681.3
CHIM-EN049,I:P Masquerena,2,Ultra Rare,42.09
```

**With incomplete input and minimum value arbitration (`-a Min` is optional)**
```
yugioh_prices_csv -f examples/example_min_arb.csv -a Min
```

Input CSV:
```
tag,name,count,rarity,price
CHIM-EN049,I:P Masquerena,1,Starlight Rare,681.3
CHIM-EN049,I:P Masquerena,2,Ultra Rare,42.09
```

Output CSV:
```
name,tag,count,rarity,price
I:P Masquerena,CHIM-EN049,,Ultra Rare,33.92
Blue-Eyes White Dragon,SDBE-EN001,,Ultra Rare,1.35
```

**With incomplete input and maximum value arbitration**
```
yugioh_prices_csv -f examples/example_max_arb.csv -a Max
```

Input CSV:
```
name
I:P Masquerena
Blue-Eyes White Dragon
```

Output CSV:
```
name,tag,count,rarity,price
I:P Masquerena,CHIM-EN049,,Starlight Rare,570.19
Blue-Eyes White Dragon,GLD5-EN001,,Ghost/Gold Rare,740.24
```

# TODOs

1. Graceful Error Handling. Currently the app expects that input is perfectly valid and it will terminate with ambiguous error 
   message if either name, tag or rarity is wrong.
2. More arbitration strategies.
3. (Maybe) - Allow using ygopro deck files(*.ydk) as input. This might be difficult.

# Acknowledgements
1. [YugiohPrices.com](http://yugiohprices.com) - Thanks for developing the awesome website and allowing us to use your API.