# Description

A very basic crawler for Myanimelist that prints out which animes are going to air
today, given the user name and date format. The reason why we ask for a date format
is because, for some reason, Myanimelist employs user specific date format in
user/animelist page. Hence, we have to prompt user for the corresponding date
format.

# Usage

`cargo run`

# Limitations

Naive HTML parser to fetch airing hour information from anime pages is way too 
slow. We need to improve that.

Using `reqwest` for a basic GET request is not acceptable. We need to write
a simple GET function.
