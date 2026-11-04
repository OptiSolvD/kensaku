import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import LanguageDetector from "i18next-browser-languagedetector";

import enCommon from "../locales/en/common.json";
import enSearch from "../locales/en/search.json";
import enHome from "../locales/en/home.json";
import enResults from "../locales/en/results.json";
import enDetails from "../locales/en/details.json";
import enHelp from "../locales/en/help.json";
import enSettings from "../locales/en/settings.json";
import enSetup from "../locales/en/setup.json";
import enErrors from "../locales/en/errors.json";

import zhCommon from "../locales/zh/common.json";
import zhSearch from "../locales/zh/search.json";
import zhHome from "../locales/zh/home.json";
import zhResults from "../locales/zh/results.json";
import zhDetails from "../locales/zh/details.json";
import zhHelp from "../locales/zh/help.json";
import zhSettings from "../locales/zh/settings.json";
import zhSetup from "../locales/zh/setup.json";
import zhErrors from "../locales/zh/errors.json";

export const SUPPORTED_LANGUAGES = {
  en: "English",
  zh: "中文",
} as const;

export type SupportedLanguage = keyof typeof SUPPORTED_LANGUAGES;

const resources = {
  en: {
    common: enCommon,
    search: enSearch,
    home: enHome,
    results: enResults,
    details: enDetails,
    help: enHelp,
    settings: enSettings,
    setup: enSetup,
    errors: enErrors,
  },
  zh: {
    common: zhCommon,
    search: zhSearch,
    home: zhHome,
    results: zhResults,
    details: zhDetails,
    help: zhHelp,
    settings: zhSettings,
    setup: zhSetup,
    errors: zhErrors,
  },
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: "en",
    supportedLngs: Object.keys(SUPPORTED_LANGUAGES),
    defaultNS: "common",
    ns: ["common"],
    interpolation: { escapeValue: false },
    detection: {
      order: ["localStorage", "navigator"],
      caches: ["localStorage"],
      lookupLocalStorage: "kensaku.language",
    },
    react: { useSuspense: false },
  });

export default i18n;
