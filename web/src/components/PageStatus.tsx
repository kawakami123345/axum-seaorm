import RequestStatus, { type RequestStatusProps } from "./RequestStatus";

export default function PageStatus(props: RequestStatusProps) {
  return (
    <main className="flex-1 min-w-0 flex items-center justify-center h-64">
      <RequestStatus {...props} />
    </main>
  );
}
