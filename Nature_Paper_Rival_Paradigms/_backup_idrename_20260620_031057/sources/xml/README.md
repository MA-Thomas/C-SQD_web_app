# Source XML

This folder stores public XML source records used to generate review PDFs when a native publisher PDF could not be downloaded as a verified PDF in this environment.

- `C0002.xml`: PMC full-text XML for `PMC9812260`.
- `C0003.xml`: PMC full-text XML for `PMC6642641`.
- `D0003.xml`: PMC full-text XML for `PMC4481139`.
- `B0003.xml`: Elsevier metadata-only XML retained as an audit artifact; it was not sufficient to generate a full paper PDF.

Generated PDFs are marked in `../acquisition_log.csv` and `../../metadata/papers.csv` with `generated_pdf_from_pmc_xml`.
