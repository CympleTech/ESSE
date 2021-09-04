String betterPrint(String? info) {
  if (info == null) {
    return '';
  }
  final len = info.length;
  if (len > 8) {
    return info.substring(0, 8) + '...' + info.substring(len - 6, len);
  } else {
    return info;
  }
}
