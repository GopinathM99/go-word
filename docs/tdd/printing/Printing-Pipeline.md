# Printing and Print Preview Pipeline

## Goals
- Match PDF export and print output.
- Provide accurate print preview.

## Print Preview
- Use same layout engine as PDF/export.
- Render pages at screen resolution.

## Printing
- Render at printer DPI.
- Use vector output when supported by printer driver.

## Margins and Page Setup
- Respect printer hardware margins.
- If margins exceed printer limits, warn user.

## Page Range and Scaling
- Support page ranges (e.g., 1-3, 5).
- Support scaling (fit to page, custom %).

## Headers/Footers
- Preserve header/footer positions at print time.

## Duplex
- Support duplex printing where available.
- Adjust margins for binding if enabled.
