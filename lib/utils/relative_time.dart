import 'package:intl/intl.dart';

class RelativeTime {
  DateTime time;

  RelativeTime(): this.time = new DateTime.now();

  RelativeTime.fromString(String datetime):
  this.time = DateFormat('yyyy-MM-dd H:mm').parse(datetime);

  RelativeTime.fromInt(int datetime):
    this.time = DateTime.fromMillisecondsSinceEpoch(datetime * 1000);

  String rawString() {
    var formatter = new DateFormat('yyyy-MM-dd H:mm:ss');
    return formatter.format(time);
  }

  String toString() {
    var now = new DateTime.now();
    if (now.year != time.year) {
      var formatter = new DateFormat('yyyy-MM-dd');
      return formatter.format(time);
    }

    if (now.day != time.day) {
      var formatter = new DateFormat('MM-dd H:mm');
      return formatter.format(time);
    }

    var formatter = new DateFormat('H:mm');
    return formatter.format(time);
  }

  bool isAfter(RelativeTime other) {
    return time.isAfter(other.time);
  }

  bool isBefore(RelativeTime other) {
    return time.isBefore(other.time);
  }

  int toInt() {
    return time.millisecondsSinceEpoch;
  }

  // [days, hours, minutes, seconds]
  List<int> uptime() {
    final now = new DateTime.now();
    Duration diff = now.difference(time);
    final days = diff.inDays;
    final hours = diff.inHours;
    final minutes = diff.inMinutes;
    final seconds = diff.inSeconds;
    return [days, hours - (days * 24), minutes - (hours * 60), seconds - (minutes * 60)];
  }
}
