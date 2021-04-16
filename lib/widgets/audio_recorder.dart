import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:record/record.dart';
import 'package:just_audio/just_audio.dart';

import 'package:esse/l10n/localizations.dart';

class AudioRecorder extends StatefulWidget {
  final String path;
  final Function onStop;

  const AudioRecorder({Key key, this.path, this.onStop}) : super(key: key);

  @override
  _AudioRecorderState createState() => _AudioRecorderState();
}

class _AudioRecorderState extends State<AudioRecorder> {
  final player = AudioPlayer();

  bool _isRecording = false;
  bool _isPlaying = false;
  bool _isPlayPause = false;
  int _remainingDuration = -1;
  Timer _timer;

  Widget _buildText(color, lang) {
    if (_remainingDuration >= 0) {
      return _buildTimer(color);
    }

    return Text(lang.waitingRecord);
  }

  Widget _buildTimer(color) {
    final String minutes = _formatNumber(_remainingDuration ~/ 60);
    final String seconds = _formatNumber(_remainingDuration % 60);

    return Text(
      '$minutes : $seconds',
      style: TextStyle(color: color),
    );
  }

  String _formatNumber(int number) {
    String numberStr = number.toString();
    if (number < 10) {
      numberStr = '0' + numberStr;
    }

    return numberStr;
  }

  Future<void> _start() async {
    try {
      if (await Record.hasPermission()) {
        await Record.start(path: widget.path);

        bool isRecording = await Record.isRecording();
        setState(() {
          _isRecording = isRecording;
          _remainingDuration = 0;
        });

        _startTimer();
      }
    } catch (e) {
      print(e);
    }
  }

  Future<void> _stop() async {
    _timer?.cancel();
    await Record.stop();
    print(widget.path);

    setState(() {
      _isRecording = false;
    });
  }

  void _startTimer() {
    const tick = const Duration(milliseconds: 500);

    _timer?.cancel();

    _timer = Timer.periodic(tick, (Timer t) async {
      if (!_isRecording) {
        t.cancel();
      } else {
        setState(() {
          _remainingDuration = (t.tick / 2).floor();
        });
      }
    });
  }

  void _pause() async {
    await player.pause();
    setState(() {
      _isPlaying = false;
      _isPlayPause = true;
    });
  }

  void _play() async {
    if (!_isPlayPause) {
      await player.setFilePath(widget.path);
    }
    print(widget.path);
    player.play();
    setState(() {
      _isPlayPause = false;
      _isPlaying = true;
    });
  }

  void _clear() async {
    _timer?.cancel();

    // delete file.
    await File(widget.path).delete();

    setState(() {
      _isRecording = false;
      _remainingDuration = -1;
    });
  }

  void _send() async {
    _timer?.cancel();
    _isRecording = false;
    _remainingDuration = -1;
    print(widget.path);
    await player.setFilePath(widget.path);
    final time = player.duration.inSeconds + 1;
    player.dispose();
    widget.onStop(time);
  }

  @override
  void initState() {
    super.initState();
    player.playerStateStream.listen((state) {
      if (state.processingState == ProcessingState.completed) {
        setState(() {
          _isPlaying = false;
          _isPlayPause = false;
        });
      }
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final color = Theme.of(context).colorScheme;
    final lang = AppLocalizations.of(context);

    return Column(
      children: [
      _buildText(color.primary, lang),
      SizedBox(height: 10.0),
      Row(mainAxisAlignment: MainAxisAlignment.center,
        children: [
          _remainingDuration > -1
          ? TextButton(onPressed: _clear, child: Text(lang.cancel))
          : const TextButton(onPressed: null, child: Text('')),
          const SizedBox(width: 20.0),
          ClipOval(
            child: Container(
              width: 60.0,
              height: 60.0,
              decoration: BoxDecoration(
                color: (_isRecording || _isPlaying)
                ? color.primary
                : color.primaryVariant,
              ),
              child: (_remainingDuration < 0 || _isRecording)
              ? GestureDetector(
                onTap: _isRecording ? _stop : _start,
                child: Icon(
                  _isRecording ? Icons.mic_rounded : Icons.mic_none_rounded,
                  color: _isRecording ? color.primaryVariant : color.primary,
                  size: _isRecording ? 48.0 : 36.0,
              ))
              : GestureDetector(
                onTap: _isPlaying ? _pause : _play,
                child: Icon(
                  _isPlaying ? Icons.pause_rounded : Icons.play_arrow_rounded,
                  color: _isPlaying ? color.primaryVariant : color.primary,
                  size: 42.0,
              )),
          )),
          const SizedBox(width: 20.0),
          (_remainingDuration > -1 && !_isRecording)
          ? TextButton(onPressed: _send, child: Text(lang.send))
          : const TextButton(onPressed: null, child: Text('')),
      ])
    ]);
  }
}
