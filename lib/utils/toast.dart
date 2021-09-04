import 'package:flutter/material.dart';

class FadeToast extends StatefulWidget {
  final Widget child;

  FadeToast({Key? key, required this.child}) : super(key: key);

  @override
  _FadeToastState createState() => _FadeToastState();
}

class _FadeToastState extends State<FadeToast>
    with SingleTickerProviderStateMixin {
  AnimationController? _controller;
  Animation<double>? _animation;

  @override
  void initState() {
    _controller =
        AnimationController(vsync: this, duration: Duration(seconds: 1));
    _animation = Tween(begin: 0.0, end: 1.0).animate(_controller!);
    _animation!.addStatusListener((status) {
      if (status == AnimationStatus.completed) {
        Future.delayed(Duration(seconds: 2), () {
          _controller!.reverse();
        });
      }
    });
    _controller!.forward();
    super.initState();
  }

  @override
  void dispose() {
    _controller!.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return FadeTransition(
      opacity: _animation!,
      child: widget.child,
    );
  }
}

void toast(BuildContext context, String text) {
  final color = Theme.of(context).colorScheme;

  ScaffoldMessenger.of(context).showSnackBar(SnackBar(
    content: FadeToast(
        child: Container(
      decoration: BoxDecoration(
          color: Color(0xFFADB0BB).withOpacity(0.5),
          borderRadius: BorderRadius.circular(25.0)),
      height: 50.0,
      margin: EdgeInsets.only(bottom: 40.0),
      alignment: Alignment.center,
      padding: const EdgeInsets.all(10.0),
      child: Text(text, style: TextStyle(color: color.onPrimary)),
    )),
    backgroundColor: Colors.transparent,
    elevation: 1000,
    behavior: SnackBarBehavior.floating,
    duration: Duration(seconds: 4),
  ));
}
