import 'dart:async';
import 'dart:ui';

import 'package:flutter/services.dart' show rootBundle;

enum AnswerLang {
  CHINESE_SIMPLIFIED,
  ENGLISH,
}

final ANSWER_LANGS = [
  AnswerLang.ENGLISH,
  AnswerLang.CHINESE_SIMPLIFIED,
];

final _langCache = Map<AnswerLang, List<String>>();

String _getAnswerLangName(AnswerLang lang) {
  switch (lang) {
    case AnswerLang.CHINESE_SIMPLIFIED:
      return 'chinese_simplified';
    case AnswerLang.ENGLISH:
      return 'english';
    default:
      return 'english';
  }
}

Future<List<String>> loadAnswers(Locale locale) async {
  AnswerLang lang = AnswerLang.ENGLISH;
  switch (locale.languageCode) {
    case 'en':
      lang = AnswerLang.ENGLISH;
      break;
    case 'zh':
      lang = AnswerLang.CHINESE_SIMPLIFIED;
      break;
    default:
      lang = AnswerLang.ENGLISH;
      break;
  }

  if (_langCache.containsKey(lang)) {
    return _langCache[lang]!;
  } else {
    final rawWords = await rootBundle.loadString('assets/answers/${_getAnswerLangName(lang)}.txt');
    final result = rawWords.split('\n').map((s) => s.trim()).where((s) => s.isNotEmpty).toList(growable: false);
    _langCache[lang] = result;
    return result;
  }
}
