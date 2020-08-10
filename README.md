# Yugioh Prices CSV
Simple command line tool to price your Yu-Gi-Oh cards. Input and output is in the form of CSV file. 
Prices are retrieved from [YugiohPrices](http://yugiohprices.com).

# Use-Case
1. Tracking value of opened booster boxes/cases
2. Tracking value of your deck

# Command Line Arguments
1. `-f <path/to/file>` - (Optional) path to input file. If not present, it defaults to STDIN.
2. `-o <path/to/file>` - (Optional) path to output file. If not present, it defaults to STDOUT.
3. `--print-total` - (Optional) flag to print the card's total value to STDOUT.

# Input/Output Data Format
CSV is the input/output data format. Headers are mandatory.

Required fields:

1. tag - The card's **print tag** (LOB-EN001, CHIM-EN049, etc)
2. name - The **name** of the card (Blue-Eyes White Dragon, I:P Masquerena, etc)

Optional fields:

1. rarity - In the case that a card's tag is ambiguous, rarity is used for arbitration, otherwise first result is used. An example 
   would be I:P Masquerena (CHIM-EN049) that comes in either Ultra Rare, or Starlight Rare
2. count - The amount of the card, defaults to 1. This value is used in calculation of total value
3. price - Price of card, the value is not used for input. This just means that you can use the output as future input (i.e. when 
   price changes the next day)

Example Input CSV:
```
tag,name,count,rarity,
CHIM-EN049,I:P Masquerena,1,Starlight Rare,
CHIM-EN049,I:P Masquerena,2,Ultra Rare,
```

Example Output CSV:
```
tag,name,count,rarity,price
CHIM-EN049,I:P Masquerena,1,Starlight Rare,681.3
CHIM-EN049,I:P Masquerena,2,Ultra Rare,42.09
```

# TODOs

1. Graceful Error Handling. Currently the app expects that input is perfectly valid and it will terminate with ambiguous error 
   message if either name, tag or rarity is wrong.
2. Allow leading/trailing white space.
3. Make tag field optional and add an option to choose tag based on price/rarity (i.e. min/max value).
4. (Maybe) - Allow using ygopro deck files(*.ydk) as input

# Acknowledgements
1. [YugiohPrices.com](http://yugiohprices.com) - Thanks for developing the awesome website and allowing us to use your API.