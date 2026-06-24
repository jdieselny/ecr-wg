# RITUAL: RENDER_DOCUMENT
# STATUS: SPECIFICATION & EXECUTABLE
# VERSION: 1.0

This ritual specifies how to translate a plain-text Markdown specification in the ECR-WG repository into a styled, professional Word document (.docx) conforming to the ECR-WG Brand Rules.

## Prerequisite Inputs

1. **Source Markdown (.md)**: The raw text specification or paper.
2. **Brand Rules (brand.md)**: The visual guidelines defining color codes, fonts, and page layouts.
3. **Format Style**: Document parameters (letter size, margins, header and footer configuration).

## Visual Styling Rules

* **Title and Heading 1 (H1)**: ECR_INDIGO (#4F46E5), Segoe UI, 18pt, bold, space before 18pt, space after 12pt.
* **Heading 2 (H2)**: ECR_SLATE (#475569), Segoe UI, 14pt, bold, space before 14pt, space after 8pt.
* **Heading 3 (H3)**: ECR_SLATE (#475569), Segoe UI, 12pt, bold, space before 12pt, space after 6pt.
* **Body Text**: ECR_CHARCOAL (#0F172A), Segoe UI, 11pt, line spacing 1.15, space before 6pt, space after 6pt.
* **Code Blocks**: Consolas, 9.5pt, ECR_SLATE color, indented 0.5 inches left.
* **Blockquotes**: Segoe UI, Italic, 10.5pt, ECR_SLATE color, indented 0.4 inches left.
* **Header & Footer**: Segoe UI, 9pt, ECR_SLATE color. Header carries document title and status. Footer carries dynamic page numbers and the public repository link.

## Executing the Ritual

Run the rendering script from the repository root:

```bash
python rituals/render_document.py [input_file.md] [output_file.docx]
```

By default, executing the script without arguments converts:
* Input: `thesis/cognitive_hypervisor_thesis.md`
* Output: `thesis/cognitive_hypervisor_thesis.docx`
