---
description: Compress and serve images in modern formats for faster loading
---

1. **Use Next.js Image Component**:
   - Automatic optimization and lazy loading.
   ```tsx
   import Image from 'next/image';
   
   <Image
     src="/hero.jpg"
     alt="Hero"
     width={1200}
     height={600}
     priority // for above-the-fold images
     placeholder="blur" // optional: shows blur while loading
   />
   ```

2. **Configure Remote Patterns (Next.js 14+)**:
   - `domains` is deprecated. Use `remotePatterns` for better security.
   ```js
   module.exports = {
     images: {
       remotePatterns: [
         {
           protocol: 'https',
           hostname: 'cdn.example.com',
           pathname: '/images/**',
         },
       ],
       formats: ['image/webp', 'image/avif'],
     },
   };
   ```

3. **Modern Formats (AVIF)**:
   - Enable AVIF for smaller file sizes (20% smaller than WebP).
   - Next.js automatically handles format negotiation.

4. **Pro Tips**:
   - Always specify width/height to prevent CLS.
   - Use `fill` + `object-fit` for responsive containers.
   - Use `sizes` prop for responsive images: `sizes="(max-width: 768px) 100vw, 50vw"`.