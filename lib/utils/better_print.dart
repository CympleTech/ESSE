String pidPrint(String? pid, [int n = 6]) {
  if (pid == null) {
    return '';
  }

  final info = pid;
  final len = info.length;
  if (len > n+n) {
    return info.substring(0, n) + '...' + info.substring(len - n, len);
  } else {
    return info;
  }
}
