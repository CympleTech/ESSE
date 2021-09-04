import 'dart:async';

import 'package:flutter/material.dart';
import 'package:just_audio/just_audio.dart';

class RecordPlayer extends StatefulWidget {
  final String path;
  final int time;

  const RecordPlayer({Key? key, required this.path, required this.time}) : super(key: key);

  @override
  _RecordPlayerState createState() => _RecordPlayerState();
}

class _RecordPlayerState extends State<RecordPlayer> {
  final player = AudioPlayer();

  bool _isPlaying = false;
  bool _isPlayPause = false;
  Timer? _timer;
  double _value = 0;
  double _valueStep = 0;

  void _startTimer() {
    const tick = const Duration(milliseconds: 500);

    _timer?.cancel();

    _timer = Timer.periodic(tick, (Timer t) async {
      if (!_isPlaying) {
        t.cancel();
      } else {
        setState(() {
            _value += _valueStep;
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
      _value = _valueStep;
    }
    _startTimer();
    player.play();
    setState(() {
        _isPlayPause = false;
        _isPlaying = true;
    });
  }

  @override
  void initState() {
    super.initState();
    _valueStep = 1 / (widget.time * 2);
    player.playerStateStream.listen((state) {
        if (state.processingState == ProcessingState.completed) {
          _timer?.cancel();
          setState(() {
              _isPlaying = false;
              _isPlayPause = false;
              _value = 1.0;
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

    return Row(
      children: [
        GestureDetector(
          onTap: _isPlaying ? _pause : _play,
          child: Icon(
            _isPlaying ? Icons.pause_rounded : Icons.play_arrow_rounded,
            color: color.primary,
            size: 20.0,
          ),
        ),
        SizedBox(width: 5.0),
        Expanded(
          child: LinearProgressIndicator(
            backgroundColor: Color(0x26ADB0BB),
            valueColor: AlwaysStoppedAnimation(color.primary),
            value: _value,
          )
        ),
        SizedBox(width: 5.0),
        Text('${widget.time}s', style: TextStyle(color: color.primary, fontSize: 12.0)),
      ]
    );
  }
}
