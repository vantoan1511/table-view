---
description: Audit and fix LCP, CLS, and INP issues for better ranking
---

1. **Fix LCP (Large Contentful Paint)**:
   - The largest element (usually the hero image) must load fast.
   - **Fix:** Add `priority` to your Hero image.
   ```tsx
   <Image src="/hero.png" alt="Hero" width={800} height={600} priority />
   ```

2. **Fix CLS (Cumulative Layout Shift)**:
   - Elements jumping around as they load cause CLS.
   - **Fix:** Always define `width` and `height` for images (or use `fill` with a parent container).
   - **Fix:** Reserve space for ads or dynamic content using CSS `min-height`.

3. **Optimize Fonts**:
   - Fonts loading late cause layout shifts (FOUT/FOIT).
   - **Fix:** Use `next/font` which automatically optimizes and hosts fonts.
   ```tsx
   import { Inter } from 'next/font/google';
   const inter = Inter({ subsets: ['latin'] });
   // Use inter.className in your body or layout
   ```

4. **Pro Tips**:
   - Run a **Lighthouse** audit in Chrome DevTools (Incognito mode) to get a baseline score.
   - Use `@next/third-parties` to load scripts like Google Analytics efficiently.