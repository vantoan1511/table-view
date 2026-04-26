---
description: Configure next-intl for multi-language support
---

1. **Install next-intl**:
   - Install the library.
   // turbo
   - Run `npm install next-intl`

2. **Create Messages**:
   - Create `messages/en.json` and `messages/es.json`.
   ```json
   {
     "Index": {
       "title": "Hello world!"
     }
   }
   ```

3. **Configure Middleware**:
   - Create `src/middleware.ts` to handle locale routing.
   ```ts
   import createMiddleware from 'next-intl/middleware';
   
   export default createMiddleware({
     locales: ['en', 'es', 'fr'],
     defaultLocale: 'en'
   });
   
   export const config = {
     matcher: ['/((?!api|_next|_vercel|.*\\..*).*)']
   };
   ```

4. **Create i18n Config**:
   - Create `src/i18n.ts` for message loading.
   ```ts
   import { getRequestConfig } from 'next-intl/server';
   
   export default getRequestConfig(async ({ locale }) => ({
     messages: (await import(`../messages/${locale}.json`)).default
   }));
   ```

5. **Pro Tips**:
   - Keep your translation keys organized by page or component.
   - Use the `useTranslations` hook in client components.
   - Use the `getTranslations` function in server components.