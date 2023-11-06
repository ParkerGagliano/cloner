# Cloner
This is a simple script to clone multiple repositories from an organization at once. 
Im currently learning Rust and using this idea to get some experience with basic rust concepts.

## Theoretical Usage
Have a folder with the binary build, and run it from there. This will make that folder the root of the cloned repositories.
Potential features to add would be a "pull" option to update all repositories.
This could be done on a cron job in conjunction with the stored gh token.

## Usage
You need a package read token [Create GH Token](https://github.com/settings/tokens/new)
```cloner --gh-token ghp_***********************```
On single selections use arrow keys to navigate and enter to select.
On multiple selections use space to select and enter to confirm.

## TODO
- Condition to check if windows and use windows commands instead of sh
- Improved error handling
- Pull feature
- Improve JSON parsing (indexed values have " around them)



## Done
- Loading Graphic
- Threading for cloning
- Store gh token by default