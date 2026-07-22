import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import { I18nManager } from 'react-native';
import ar from './ar';
import en from './en';

const savedLang = 'ar'; // AsyncStorage can be used to persist

i18n.use(initReactI18next).init({
  resources: { ar, en },
  lng: savedLang,
  fallbackLng: 'en',
  interpolation: { escapeValue: false },
});

export const changeLanguage = (lang: 'ar' | 'en') => {
  i18n.changeLanguage(lang);
  I18nManager.allowRTL(lang === 'ar');
  I18nManager.forceRTL(lang === 'ar');
};

export default i18n;
