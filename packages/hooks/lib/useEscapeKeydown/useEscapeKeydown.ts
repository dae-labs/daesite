"use client";

import {useEffect} from "react";
import {useCallbackRef} from "~/useCallbackRef";

/**
 * Listens for when the escape key is down
 */
const useEscapeKeydown = (
  onEscapeKeyDownProp?: (event: KeyboardEvent) => void,
  ownerDocument: Document = globalThis?.document,
) => {
  const onEscapeKeyDown = useCallbackRef(onEscapeKeyDownProp);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onEscapeKeyDown(event);
      }
    };
    ownerDocument.addEventListener("keydown", handleKeyDown, {capture: true});
    return () =>
      ownerDocument.removeEventListener("keydown", handleKeyDown, {
        capture: true,
      });
  }, [onEscapeKeyDown, ownerDocument]);
};

export {useEscapeKeydown};
