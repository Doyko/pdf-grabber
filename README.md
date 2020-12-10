# PDF Grabber

This project aims at gathering all PDF files from a list of websites.

## Compilation and execution

All targeted URLs should be specified in the file named `target.json` as following:
```json
{
    "id_1": "url_1",
    "id_2": "url_2",
    "id_3": "url_3"
}
```
Then, the program can be built and ran using cargo:
```
cargo run
```
The program will then crawl all the websites listed in `target.json` and download all the PDF files it finds.

## Output

When ran, the program creates a directory called `pdf`. Then, it creates a directory `pdf/id` where id is the identifier of the website it is crawling. All the PDF files found on this site will be downloaded in this folder.

The program also logs a trace in a file `output.log`.
