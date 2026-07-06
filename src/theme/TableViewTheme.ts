import { definePreset } from '@primeuix/themes';
import Aura from '@primeuix/themes/aura';

export const TableViewTheme = definePreset(Aura, {
  semantic: {
    primary: {
      50: '{indigo.50}',
      100: '{indigo.100}',
      200: '{indigo.200}',
      300: '{indigo.300}',
      400: '{indigo.400}',
      500: '{indigo.500}',
      600: '{indigo.600}',
      700: '{indigo.700}',
      800: '{indigo.800}',
      900: '{indigo.900}',
      950: '{indigo.950}'
    },
    colorScheme: {
      light: {
        surface: {
          0: '#ffffff',
          50: '{zinc.50}',
          100: '{zinc.100}',
          200: '{zinc.200}',
          300: '{zinc.300}',
          400: '{zinc.400}',
          500: '{zinc.500}',
          600: '{zinc.600}',
          700: '{zinc.700}',
          800: '{zinc.800}',
          900: '{zinc.900}',
          950: '{zinc.950}'
        }
      },
      dark: {
        surface: {
          0: '#1e1e2e',
          50: '#181825',
          100: '#1e1e2e',
          200: '#313244',
          300: '#45475a',
          400: '#585b70',
          500: '#6c7086',
          600: '#a6adc8',
          700: '#bac2de',
          800: '#cdd6f4',
          900: '#cdd6f4',
          950: '#181825'
        }
      }
    }
  }
});
