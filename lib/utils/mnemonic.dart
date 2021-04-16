import 'dart:async';
import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import 'package:flutter/services.dart' show rootBundle;

import 'package:convert/convert.dart';
import 'package:crypto/crypto.dart';
//import 'package:pointycastle/pointycastle.dart';
import 'package:unorm_dart/unorm_dart.dart';

import '../l10n/localizations.dart';

enum MnemonicLang {
  CHINESE_SIMPLIFIED,
  CHINESE_TRADITIONAL,
  ENGLISH,
  FRENCH,
  ITALIAN,
  JAPANESE,
  KOREAN,
  SPANISH,
}

final MNEMONIC_LANGS = [
  MnemonicLang.ENGLISH,
  MnemonicLang.CHINESE_SIMPLIFIED,
];

extension MnemonicLangExtension on MnemonicLang {
  String localizations(context) {
    final lang = AppLocalizations.of(context);
    switch (this) {
      case MnemonicLang.CHINESE_SIMPLIFIED:
        return lang.langZh;
      case MnemonicLang.ENGLISH:
        return lang.langEn;
      default:
        return lang.langEn;
    }
  }

  int toInt() {
    switch (this) {
      case MnemonicLang.ENGLISH:
        return 0;
      case MnemonicLang.CHINESE_SIMPLIFIED:
        return 1;
      default:
        return 0;
    }
  }

  static MnemonicLang fromInt(int a) {
    switch (a) {
      case 0:
        return MnemonicLang.ENGLISH;
      case 1:
        return MnemonicLang.CHINESE_SIMPLIFIED;
      default:
        return MnemonicLang.ENGLISH;
    }
  }
}

final _langCache = Map<MnemonicLang, List<dynamic>>();

const MnemonicLang _DEFAULT_LANG = MnemonicLang.ENGLISH;

const int _SIZE_8BITS = 255;
const String _INVALID_ENTROPY = 'Invalid entropy';
const String _INVALID_MNEMONIC = 'Invalid mnemonic';
const String _INVALID_CHECKSUM = 'Invalid checksum';

String _getMnemonicLangName(MnemonicLang lang) {
  switch (lang) {
    case MnemonicLang.CHINESE_SIMPLIFIED:
      return 'chinese_simplified';
    case MnemonicLang.CHINESE_TRADITIONAL:
      return 'chinese_traditional';
    case MnemonicLang.ENGLISH:
      return 'english';
    case MnemonicLang.FRENCH:
      return 'french';
    case MnemonicLang.ITALIAN:
      return 'italian';
    case MnemonicLang.JAPANESE:
      return 'japanese';
    case MnemonicLang.KOREAN:
      return 'korean';
    case MnemonicLang.SPANISH:
      return 'spanish';
    default:
      return 'english';
  }
}

int _binaryToByte(String binary) {
  return int.parse(binary, radix: 2);
}

String _bytesToBinary(Uint8List bytes) {
  return bytes.map((byte) => byte.toRadixString(2).padLeft(8, '0')).join('');
}

String _salt(String password) {
  return 'mnemonic${password ?? ""}';
}

String _deriveChecksumBits(Uint8List entropy) {
  final ENT = entropy.length * 8;
  final CS = ENT ~/ 32;

  final hash = sha256.convert(entropy);
  return _bytesToBinary(Uint8List.fromList(hash.bytes)).substring(0, CS);
}

typedef Uint8List RandomBytes(int size);

Uint8List _nextBytes(int size) {
  final rnd = Random.secure();
  final bytes = Uint8List(size);
  for (var i = 0; i < size; i++) {
    bytes[i] = rnd.nextInt(_SIZE_8BITS);
  }
  return bytes;
}

/// Converts [mnemonic] code to entropy.
Future<Uint8List> mnemonicToEntropy(String mnemonic,
    [MnemonicLang lang = _DEFAULT_LANG]) async {
  final wordRes = await _loadMnemonicLang(lang);
  final words = nfkd(mnemonic).split(' ');

  if (words.length % 3 != 0) {
    throw new ArgumentError(_INVALID_MNEMONIC);
  }

  // convert word indices to 11bit binary strings
  final bits = words.map((word) {
    final index = wordRes.indexOf(word);
    if (index == -1) {
      throw ArgumentError(_INVALID_MNEMONIC);
    }

    return index.toRadixString(2).padLeft(11, '0');
  }).join('');

  // split the binary string into ENT/CS
  final dividerIndex = (bits.length / 33).floor() * 32;
  final entropyBits = bits.substring(0, dividerIndex);
  final checksumBits = bits.substring(dividerIndex);

  final regex = RegExp(r".{1,8}");

  final entropyBytes = Uint8List.fromList(regex
      .allMatches(entropyBits)
      .map((match) => _binaryToByte(match.group(0)))
      .toList(growable: false));
  if (entropyBytes.length < 16) {
    throw StateError(_INVALID_ENTROPY);
  }
  if (entropyBytes.length > 32) {
    throw StateError(_INVALID_ENTROPY);
  }
  if (entropyBytes.length % 4 != 0) {
    throw StateError(_INVALID_ENTROPY);
  }

  final newCheckSum = _deriveChecksumBits(entropyBytes);
  if (newCheckSum != checksumBits) {
    throw StateError(_INVALID_CHECKSUM);
  }

  return entropyBytes;
}

/// Converts [entropy] to mnemonic code.
Future<String> entropyToMnemonic(Uint8List entropy,
    [MnemonicLang lang = _DEFAULT_LANG]) async {
  if (entropy.length < 16) {
    throw ArgumentError(_INVALID_ENTROPY);
  }
  if (entropy.length > 32) {
    throw ArgumentError(_INVALID_ENTROPY);
  }
  if (entropy.length % 4 != 0) {
    throw ArgumentError(_INVALID_ENTROPY);
  }

  final entropyBits = _bytesToBinary(entropy);
  final checksumBits = _deriveChecksumBits(entropy);

  final bits = entropyBits + checksumBits;

  final regex = new RegExp(r".{1,11}", caseSensitive: false, multiLine: false);
  final chunks = regex
      .allMatches(bits)
      .map((match) => match.group(0))
      .toList(growable: false);

  final wordRes = await _loadMnemonicLang(lang);

  return chunks
      .map((binary) => wordRes[_binaryToByte(binary)])
      .join(lang == MnemonicLang.JAPANESE ? '\u3000' : ' ');
}

/// Converts HEX string [entropy] to mnemonic code
Future<String> entropyHexToMnemonic(String entropy,
    [MnemonicLang lang = _DEFAULT_LANG]) {
  return entropyToMnemonic(hex.decode(entropy), lang);
}

/// Generates a random mnemonic.
///
/// Defaults to 128-bits of entropy.
/// By default it uses [Random.secure()] under the food to get random bytes,
/// but you can swap RNG by providing [randomBytes].
/// Default lang is English, but you can use different lang by providing [lang].
Future<String> generateMnemonic({
  int strength = 128,
  RandomBytes randomBytes = _nextBytes,
  MnemonicLang lang = _DEFAULT_LANG,
}) async {
  assert(strength % 32 == 0);

  final entropy = randomBytes(strength ~/ 8);

  return await entropyToMnemonic(entropy, lang);
}

/// Check if [mnemonic] code is valid.
Future<bool> validateMnemonic(String mnemonic,
    [MnemonicLang lang = _DEFAULT_LANG]) async {
  try {
    await mnemonicToEntropy(mnemonic, lang);
  } catch (e) {
    return false;
  }
  return true;
}

Future<List<String>> _loadMnemonicLang(MnemonicLang lang) async {
  if (_langCache.containsKey(lang)) {
    return _langCache[lang];
  } else {
    final rawWords = await rootBundle
        .loadString('assets/mnemonic/${_getMnemonicLangName(lang)}.txt');
    final result = rawWords
        .split('\n')
        .map((s) => s.trim())
        .where((s) => s.isNotEmpty)
        .toList(growable: false);
    _langCache[lang] = result;
    return result;
  }
}
