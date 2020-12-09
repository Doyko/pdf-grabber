# PDF Grabber

This project aims at gathering all pdf files from a list of websites.

## Compilation and execution

First, the targetted URL should be specified in a file named `target.json` as following:
```json
{
    "id1": "url1",
    "id2": "url2",
    ...
}
```
Then, the program can be simply built and run using cargo:
```
cargo run
```
The program will then crawl all the websites listed in `target.json` and download all the PDF files it finds.

## Output

When ran, the program creates a directory called `pdf` where it is ran. Then, it creates a directory `pdf/id` where `id` is the identifier of the website it is crawling at the moment. All the found PDF files will be downloaded in `pdf/id/`.

The program also logs its findings in a file `output.log`.