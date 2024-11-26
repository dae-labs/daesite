"use client";

import "./StarsBackground.scss";

import stars from "@daesite/assets/video/fullscreen/stars.webm";

const StarsBackground = () => {
  return (
    <video className="stars-wrapper" autoPlay loop muted playsInline>
      <source src={stars} type="video/webm" />
    </video>
  );
};

export {StarsBackground};
