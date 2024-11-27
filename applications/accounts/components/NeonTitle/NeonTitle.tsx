import "./NeonTitle.scss";

import {HTMLProps, ReactNode} from "react";

export interface NeonTitleProps extends HTMLProps<HTMLDivElement> {
  children: ReactNode;
}

const NeonTitle = ({children, ...restProps}: NeonTitleProps) => {
  return (
    <div {...restProps}>
      <h1 className="epic-title">{children}</h1>
    </div>
  );
};

export {NeonTitle};
