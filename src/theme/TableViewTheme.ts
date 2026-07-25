import { definePreset } from '@primeuix/themes';

import Aura from '@primeuix/themes/aura';

export const TableViewTheme = definePreset(Aura, {
  semantic: {
    focusRing: {
      width: '2px',
      style: 'dashed',
      color: '{primary.color}',
      offset: '5px'
    },
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
          0: '#ffffff',
          50: '{gray.50}',
          100: '{gray.100}',
          200: '{gray.200}',
          300: '{gray.300}',
          400: '{gray.400}',
          500: '{gray.500}',
          600: '{gray.600}',
          700: '{gray.700}',
          800: '{gray.800}',
          900: '{gray.900}',
          950: '{gray.950}'
        }
      }
    }
  },
  components: {
    button: {
      colorScheme: {
        dark: {
          text: {
            secondary: {
              hoverBackground: 'var(--color-primary-light)',
              activeBackground: 'var(--color-primary-light)'
            }
          },
          root: {
            primary: {
              activeBackground: 'var(--color-primary-hover)',
              hoverBackground: 'var(--color-primary-hover)',
              hoverBorderColor: 'var(--color-border-strong)',
              activeBorderColor: 'var(--color-border-strong)'
            }
          }
        },
        light: {
          text: {
            secondary: {
              hoverBackground: '{slate.100}',
              activeBackground: '{slate.200}'
            }
          }
        }
      }
    },
    toggleswitch: {
      colorScheme: {
        dark: {
          root: {
            background: 'var(--color-primary-light)',
            hoverBackground: 'var(--color-primary-light)'
          }
        }
      }
    }
  }
});
