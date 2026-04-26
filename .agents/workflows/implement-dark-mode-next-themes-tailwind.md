---
description: Add dark mode support using next-themes
---

1. **Install next-themes**:
   - Install the library.
   // turbo
   - Run `npm install next-themes`

2. **Add Provider**:
   - Wrap your app in `app/layout.tsx`.
   ```tsx
   import { ThemeProvider } from 'next-themes';
   
   export default function RootLayout({ children }: { children: React.ReactNode }) {
     return (
       <html lang="en" suppressHydrationWarning>
         <body>
           <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
             {children}
           </ThemeProvider>
         </body>
       </html>
     );
   }
   ```

3. **Configure Tailwind**:
   - Ensure `darkMode: 'class'` is in `tailwind.config.ts`.
   ```ts
   export default {
     darkMode: 'class',
     // ... rest of config
   }
   ```

4. **Create Toggle Button**:
   - Build a theme switcher component.
   ```tsx
   'use client'
   import { useTheme } from 'next-themes';
   import { useEffect, useState } from 'react';
   
   export function ThemeToggle() {
     const [mounted, setMounted] = useState(false);
     const { theme, setTheme } = useTheme();
     
     useEffect(() => setMounted(true), []);
     if (!mounted) return null;
     
     return (
       <button onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
         {theme === 'dark' ? '🌞' : '🌙'}
       </button>
     );
   }
   ```

5. **Pro Tips**:
   - Use `suppressHydrationWarning` on the `<html>` tag to prevent hydration warnings.
   - The `useEffect` check ensures the component only renders after hydration.