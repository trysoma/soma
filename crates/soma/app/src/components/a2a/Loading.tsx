import React from "react";

interface LoadingProps {
  text?: string;
}

export const Loading: React.FC<LoadingProps> = ({ text = "Loading" }) => {
  const [loadingDots, setLoadingDots] = React.useState<string>(".");

  React.useEffect(() => {
    const interval = setInterval(() => {
      setLoadingDots((prev) => {
        switch (prev) {
          case "":
            return ".";
          case ".":
            return "..";
          case "..":
            return "...";
          default:
            return "";
        }
      });
    }, 500);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex items-center justify-start gap-2">
      <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent" />
      <span className="text-sm">
        {text}
        {loadingDots}
      </span>
    </div>
  );
};
