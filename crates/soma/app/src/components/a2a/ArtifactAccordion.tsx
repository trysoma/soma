import type { Artifact, Part } from "@a2a-js/sdk";
import React from "react";

import { TextDataPartMarkdown } from "@/components/a2a/TextDataPartMarkdown";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";

interface ArtifactAccordionProps {
  artifact: Artifact;
}

const renderPart = (part: Part, index: number): React.ReactNode => {
  if (part.kind === "text") {
    return (
      <div key={index} className="pb-2">
        <TextDataPartMarkdown key={index} part={part} />
      </div>
    );
  } else if (part.kind === "data") {
    return <TextDataPartMarkdown key={index} part={part} />;
  } else {
    return null;
  }
};

export const ArtifactAccordion: React.FC<ArtifactAccordionProps> = ({ artifact }) => {
  return (
    <Accordion type="single" collapsible className="border rounded-lg">
      <AccordionItem value="item-1" className="border-none">
        <AccordionTrigger className="px-4 hover:no-underline">
          <div>
            <p className="text-sm mb-1">Artifact {artifact.artifactId}</p>

            {artifact.name && (
              <h3 className="text-2xl font-bold mb-1">{artifact.name}</h3>
            )}

            {artifact.description && (
              <p className="text-muted-foreground">{artifact.description}</p>
            )}
          </div>
        </AccordionTrigger>

        <AccordionContent className="px-4 pb-4">
          {artifact.parts.map((part, index) => renderPart(part, index))}
        </AccordionContent>
      </AccordionItem>
    </Accordion>
  );
};
