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
          50: '{slate.50}',
          100: '{slate.100}',
          200: '{slate.200}',
          300: '{slate.300}',
          400: '{slate.400}',
          500: '{slate.500}',
          600: '{slate.600}',
          700: '{slate.700}',
          800: '{slate.800}',
          900: '{slate.900}',
          950: '{slate.950}'
        }
      }
    }
  },
  components: {
    select: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            hoverBorderColor: 'var(--color-border-strong)',
            focusBorderColor: 'var(--color-primary)'
          },
          dropdown: {
            color: 'var(--color-text-tertiary)'
          },
          option: {
            focusBackground: 'var(--color-hover)',
            selectedBackground: 'var(--color-primary-light)',
            selectedColor: 'var(--color-primary)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            hoverBorderColor: 'var(--color-border-strong)',
            focusBorderColor: 'var(--color-primary)'
          },
          dropdown: {
            color: 'var(--color-text-tertiary)'
          },
          option: {
            focusBackground: 'var(--color-hover)',
            selectedBackground: 'var(--color-primary-light)',
            selectedColor: 'var(--color-primary)'
          }
        }
      }
    },
    popover: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)'
          }
        }
      }
    },
    dialog: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          }
        }
      }
    },
    inputtext: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            hoverBorderColor: 'var(--color-border-strong)',
            focusBorderColor: 'var(--color-primary)',
            color: 'var(--color-text-primary)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            hoverBorderColor: 'var(--color-border-strong)',
            focusBorderColor: 'var(--color-primary)',
            color: 'var(--color-text-primary)'
          }
        }
      }
    },
    contextmenu: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          },
          item: {
            focusBackground: 'var(--color-hover)',
            color: 'var(--color-text-primary)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          },
          item: {
            focusBackground: 'var(--color-hover)',
            color: 'var(--color-text-primary)'
          }
        }
      }
    },
    menu: {
      colorScheme: {
        light: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          },
          item: {
            focusBackground: 'var(--color-hover)',
            color: 'var(--color-text-primary)'
          }
        },
        dark: {
          root: {
            background: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text-primary)'
          },
          item: {
            focusBackground: 'var(--color-hover)',
            color: 'var(--color-text-primary)'
          }
        }
      }
    }
  }
});
